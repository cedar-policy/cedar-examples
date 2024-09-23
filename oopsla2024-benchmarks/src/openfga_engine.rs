use super::{utils, ExampleApp, SingleExecutionReport, TemplateLink};
use cedar_policy_core::ast::{Eid, Entity, EntityType, EntityUID, EntityUIDEntry, Request};
use cedar_policy_core::authorizer::Decision;
use std::collections::HashMap;
use std::fs::File;
use std::process::{Child, Command};
use std::time::Duration;

mod client;
use client::{OpenFgaClient, OpenFgaTuple};

const OPENFGA_BIN_PATH: &str = "openfga"; // assume it is on PATH

pub struct OpenFgaEngine<'a> {
    /// app object
    app: &'a ExampleApp,
    /// OpenFGA Server process -- we hold on to this so that it is `Drop`ped when this engine is dropped
    _server: OpenFgaServer,
    /// Client for talking to the OpenFGA server
    client: OpenFgaClient,
}

impl<'a> OpenFgaEngine<'a> {
    pub fn new<'b>(
        entities: impl IntoIterator<Item = &'b Entity> + 'b,
        links: impl IntoIterator<Item = TemplateLink>,
        app: &'a ExampleApp,
    ) -> Self {
        let server = OpenFgaServer::new().expect("failed to spawn OpenFGA server");
        let mut client =
            OpenFgaClient::new("http://localhost:8080", app.openfga_authz_model_filename);
        {
            match app.name {
                "github" => {
                    let ghconverter = GithubConverter::new(app);
                    let adds = ghconverter.convert_entities(entities);
                    client.add_tuples(adds);
                }
                "github-templates" => {
                    let ghconverter = GithubConverter::new(app);
                    let mut adds = ghconverter.convert_entities(entities);
                    adds.extend(ghconverter.convert_links(links));
                    client.add_tuples(adds);
                }
                "gdrive" => {
                    let gdconverter = GdriveConverter::new(app);
                    let adds = gdconverter.convert_entities(entities);
                    client.add_tuples(adds);
                }
                "gdrive-templates" => {
                    let gdconverter = GdriveConverter::new(app);
                    let mut adds = gdconverter.convert_entities(entities);
                    adds.extend(gdconverter.convert_links(links));
                    client.add_tuples(adds);
                }
                "tinytodo" => {
                    let converter = TinyTodoConverter::new(app);
                    let adds = converter.convert_entities(entities);
                    client.add_tuples(adds);
                }
                app_name => unimplemented!("example app {app_name}"),
            };
        }
        Self {
            app,
            _server: server,
            client,
        }
    }

    pub fn execute(&self, request: Request) -> SingleExecutionReport {
        let tuple = OpenFgaTuple {
            user: match request.principal() {
                EntityUIDEntry::Known { euid, .. } => (self.app.convert_euid)(euid),
                EntityUIDEntry::Unknown { .. } => panic!("can't handle requests with Unknown"),
            },
            relation: match request.action() {
                EntityUIDEntry::Known { euid, .. } => (self.app.convert_euid)(euid),
                EntityUIDEntry::Unknown { .. } => panic!("can't handle requests with Unknown"),
            },
            object: match request.resource() {
                EntityUIDEntry::Known { euid, .. } => (self.app.convert_euid)(euid),
                EntityUIDEntry::Unknown { .. } => panic!("can't handle requests with Unknown"),
            },
        };
        let allowed = self.client.check(&tuple);
        let f = File::open("/tmp/openfga_times").expect("file should exist");
        let dur = {
            let m: HashMap<String, u64> =
                serde_json::from_reader(f).expect("failed to read from file");
            Duration::from_micros(
                *m.get("check_time_micros")
                    .expect("expected check_time_micros key"),
            )
        };
        SingleExecutionReport {
            dur,
            decision: if allowed {
                Decision::Allow
            } else {
                Decision::Deny
            },
            errors: vec![], // all errors in OpenFGA are currently panics
            context_attrs: request
                .context()
                .cloned()
                .map(|ctx| ctx.into_iter().count())
                .unwrap_or(0),
        }
    }

    /// How many `OpenFgaTuple`s have been added
    pub fn num_tuples(&self) -> usize {
        self.client.num_tuples()
    }
}

/// Cache for entity types used in the Cedar github encodings, so we don't have
/// to keep creating them all the time
struct GithubTypes {
    user: EntityType,
    repopermission: EntityType,
    orgpermission: EntityType,
    team: EntityType,
    repo: EntityType,
    org: EntityType,
}

impl GithubTypes {
    fn new() -> Self {
        Self {
            user: utils::entity_type("User"),
            repopermission: utils::entity_type("RepoPermission"),
            orgpermission: utils::entity_type("OrgPermission"),
            team: utils::entity_type("Team"),
            repo: utils::entity_type("Repository"),
            org: utils::entity_type("Organization"),
        }
    }
}

/// Converts Cedar entity data into `OpenFgaTuple`s, for both the Github encodings
struct GithubConverter<'a> {
    app: &'a ExampleApp,
    ghtypes: GithubTypes,
}

impl<'a> GithubConverter<'a> {
    fn new(app: &'a ExampleApp) -> Self {
        Self {
            app,
            ghtypes: GithubTypes::new(),
        }
    }

    /// Convert these Cedar entities into `OpenFgaTuple`s.
    /// The number of tuples will not necessarily be the same as the number of entities.
    fn convert_entities<'b>(
        &self,
        entities: impl IntoIterator<Item = &'b Entity>,
    ) -> Vec<OpenFgaTuple> {
        entities
            .into_iter()
            .flat_map(move |entity| {
                if entity.uid().entity_type() == &self.ghtypes.user {
                    entity
                        .ancestors()
                        .map(|ancestor| {
                            if ancestor.entity_type() == &self.ghtypes.repopermission
                                || ancestor.entity_type() == &self.ghtypes.orgpermission
                            {
                                self.repo_org_permission_to_tuple(ancestor, &entity.uid())
                            } else if ancestor.entity_type() == &self.ghtypes.team
                                || ancestor.entity_type() == &self.ghtypes.org
                            {
                                self.parent_to_tuple(&entity.uid(), ancestor)
                            } else {
                                panic!("unexpected ancestor of User: {ancestor}")
                            }
                        })
                        .collect::<Vec<_>>()
                } else if entity.uid().entity_type() == &self.ghtypes.team {
                    entity
                        .ancestors()
                        .map(|ancestor| {
                            if ancestor.entity_type() == &self.ghtypes.repopermission {
                                self.repo_org_permission_to_tuple(ancestor, &entity.uid())
                            } else if ancestor.entity_type() == &self.ghtypes.team {
                                self.parent_to_tuple(&entity.uid(), ancestor)
                            } else {
                                panic!("unexpected ancestor of Team: {ancestor}")
                            }
                        })
                        .collect::<Vec<_>>()
                } else if entity.uid().entity_type() == &self.ghtypes.repo {
                    let owner = utils::pv_expect_euid(
                        entity.get("owner").expect("repo should have a .owner"),
                    );
                    vec![OpenFgaTuple {
                        user: (self.app.convert_euid)(&owner),
                        relation: "owner".to_string(),
                        object: (self.app.convert_euid)(&entity.uid()),
                    }]
                } else if entity.uid().entity_type() == &self.ghtypes.org {
                    entity
                        .ancestors()
                        .map(|ancestor| {
                            if ancestor.entity_type() == &self.ghtypes.orgpermission {
                                self.repo_org_permission_to_tuple(ancestor, &entity.uid())
                            } else {
                                panic!("unexpected ancestor of Org: {ancestor}")
                            }
                        })
                        .collect::<Vec<_>>()
                } else {
                    vec![]
                }
            })
            .collect()
    }

    /// Convert these Cedar `TemplateLink`s into `OpenFgaTuple`s
    fn convert_links(&self, links: impl IntoIterator<Item = TemplateLink>) -> Vec<OpenFgaTuple> {
        links
            .into_iter()
            .map(|link| OpenFgaTuple {
                user: {
                    let mut user = (self.app.convert_euid)(&link.principal);
                    if link.principal.entity_type() == &self.ghtypes.team
                        || link.principal.entity_type() == &self.ghtypes.org
                    {
                        user.push_str("#member");
                    }
                    user
                },
                relation: match link.template_name.as_str() {
                    "readTemplate" => "reader".to_string(),
                    "triageTemplate" => "triager".to_string(),
                    "writeTemplate" => "writer".to_string(),
                    "maintainTemplate" => "maintainer".to_string(),
                    "adminTemplate" => "admin".to_string(),
                    _ => panic!("unexpected template_name: {}", &link.template_name),
                },
                object: (self.app.convert_euid)(&link.resource),
            })
            .collect()
    }

    /// Helper function: split the EID of a RepoPermission or OrgPermission into
    /// components. For instance, for `RepoPermission::"repo_0_readers"`, return
    /// "repo_0" and the OpenFGA relation "reader"
    fn split_permission_eid(eid: &Eid) -> Option<(&str, &str)> {
        let eid: &str = eid.as_ref();
        if let Some(repo_or_org) = eid.strip_suffix("_readers") {
            Some((repo_or_org, "reader"))
        } else if let Some(repo_or_org) = eid.strip_suffix("_triagers") {
            Some((repo_or_org, "triager"))
        } else if let Some(repo_or_org) = eid.strip_suffix("_writers") {
            Some((repo_or_org, "writer"))
        } else if let Some(repo_or_org) = eid.strip_suffix("_maintainers") {
            Some((repo_or_org, "maintainer"))
        } else if let Some(repo_or_org) = eid.strip_suffix("_admins") {
            Some((repo_or_org, "admin"))
        } else {
            None
        }
    }

    /// convert a RepoPermission or OrgPermission into the corresponding OpenFGA tuple
    ///
    /// `perm` should have type `RepoPermission` or `OrgPermission`.
    /// `grantee` is the entity that has the permission.
    fn repo_org_permission_to_tuple(&self, perm: &EntityUID, grantee: &EntityUID) -> OpenFgaTuple {
        let mut user = (self.app.convert_euid)(grantee);
        if grantee.entity_type() == &self.ghtypes.team || grantee.entity_type() == &self.ghtypes.org
        {
            user.push_str("#member");
        }
        let Some((repo_or_org, relation)) = Self::split_permission_eid(perm.eid()) else {
            panic!("Unexpected parent of {grantee}: {perm}");
        };
        OpenFgaTuple {
            user,
            relation: if perm.entity_type() == &self.ghtypes.repopermission {
                format!("{relation}")
            } else {
                format!("repo_{relation}") // orgs have relations like "repo_reader" etc, while repos have "reader" etc. I didn't make the rules
            },
            object: if perm.entity_type() == &self.ghtypes.repopermission {
                format!("repo:\"{repo_or_org}\"")
            } else {
                format!("organization:\"{repo_or_org}\"")
            },
        }
    }

    /// create the OpenFGA tuple representing membership of `child` in `parent` in
    /// the Cedar entity data
    fn parent_to_tuple(&self, child: &EntityUID, parent: &EntityUID) -> OpenFgaTuple {
        let mut user = (self.app.convert_euid)(child);
        if child.entity_type() == &self.ghtypes.team || child.entity_type() == &self.ghtypes.org {
            user.push_str("#member");
        }
        OpenFgaTuple {
            user,
            relation: "member".to_string(),
            object: (self.app.convert_euid)(&parent),
        }
    }
}

struct GdriveTypes {
    user: EntityType,
    group: EntityType,
    doc: EntityType,
    folder: EntityType,
    view: EntityType,
}

impl GdriveTypes {
    fn new() -> Self {
        Self {
            user: utils::entity_type("User"),
            group: utils::entity_type("Group"),
            doc: utils::entity_type("Document"),
            folder: utils::entity_type("Folder"),
            view: utils::entity_type("View"),
        }
    }
}

/// Converts Cedar entity data into `OpenFgaTuple`s, for both the Github encodings
struct GdriveConverter<'a> {
    app: &'a ExampleApp,
    gdtypes: GdriveTypes,
}

impl<'a> GdriveConverter<'a> {
    fn new(app: &'a ExampleApp) -> Self {
        Self {
            app,
            gdtypes: GdriveTypes::new(),
        }
    }

    /// Convert these Cedar entities into `OpenFgaTuple`s.
    /// The number of tuples will not necessarily be the same as the number of entities.
    fn convert_entities<'b>(
        &self,
        entities: impl IntoIterator<Item = &'b Entity>,
    ) -> Vec<OpenFgaTuple> {
        entities
            .into_iter()
            .flat_map(move |entity| {
                if entity.uid().entity_type() == &self.gdtypes.user {
                    let groups = entity
                        .ancestors()
                        .map(|ancestor| self.parent_to_tuple(&entity.uid(), ancestor));
                    let owned_documents = utils::pv_expect_set_euids(
                        entity
                            .get("ownedDocuments")
                            .expect("user should have .ownedDocuments"),
                    )
                    .map(|owned_doc| self.owner_to_tuple(&entity.uid(), &owned_doc));
                    let owned_folders = utils::pv_expect_set_euids(
                        entity
                            .get("ownedFolders")
                            .expect("user should have .ownedFolders"),
                    )
                    .map(|owned_folder| self.owner_to_tuple(&entity.uid(), &owned_folder));
                    groups
                        .chain(owned_documents)
                        .chain(owned_folders)
                        .collect::<Vec<_>>()
                } else if entity.uid().entity_type() == &self.gdtypes.folder {
                    entity
                    .ancestors()
                    .map(|ancestor| {
                        if ancestor.entity_type() == &self.gdtypes.folder {
                            self.parent_folder(&entity.uid(), ancestor)
                        } else if ancestor.entity_type() == &self.gdtypes.view {
                            self.view_to_tuple(ancestor, &entity.uid())
                        } else {
                            panic!("Expected all ancestors of a Folder to be either Folder or View")
                        }
                    })
                    .collect::<Vec<_>>()
                } else if entity.uid().entity_type() == &self.gdtypes.doc {
                    let mut tuples = entity
                        .ancestors()
                        .map(|ancestor| {
                            if ancestor.entity_type() == &self.gdtypes.folder {
                                self.parent_folder(&entity.uid(), ancestor)
                            } else if ancestor.entity_type() == &self.gdtypes.view {
                                self.view_to_tuple(ancestor, &entity.uid())
                            } else {
                                panic!(
                                "Expected all ancestors of a Document to be either Folder or View"
                            )
                            }
                        })
                        .collect::<Vec<_>>();
                    if utils::pv_expect_bool(
                        entity
                            .get("isPublic")
                            .expect("Document should have .isPublic"),
                    ) {
                        tuples.push(self.public_view(&entity.uid()));
                    }
                    tuples
                } else {
                    vec![]
                }
            })
            .collect()
    }

    /// Convert these Cedar `TemplateLink`s into `OpenFgaTuple`s
    fn convert_links(&self, links: impl IntoIterator<Item = TemplateLink>) -> Vec<OpenFgaTuple> {
        links
            .into_iter()
            .map(|link| OpenFgaTuple {
                user: {
                    let mut user = (self.app.convert_euid)(&link.principal);
                    if link.principal.entity_type() == &self.gdtypes.group {
                        user.push_str("#member");
                    }
                    user
                },
                relation: match link.template_name.as_str() {
                    "template" => "viewer".to_string(),
                    _ => panic!("unexpected template_name: {}", &link.template_name),
                },
                object: (self.app.convert_euid)(&link.resource),
            })
            .collect()
    }

    /// create the OpenFGA tuple representing membership of `child` in `parent` in
    /// the Cedar entity data
    fn parent_to_tuple(&self, child: &EntityUID, parent: &EntityUID) -> OpenFgaTuple {
        let mut user = (self.app.convert_euid)(child);
        if child.entity_type() == &self.gdtypes.group {
            user.push_str("#member");
        }
        OpenFgaTuple {
            user,
            relation: "member".to_string(),
            object: (self.app.convert_euid)(parent),
        }
    }

    /// create the OpenFGA tuple representing that `owner` owns `owned`
    fn owner_to_tuple(&self, owner: &EntityUID, owned: &EntityUID) -> OpenFgaTuple {
        assert_eq!(owner.entity_type(), &self.gdtypes.user); // only Users can own things
        OpenFgaTuple {
            user: (self.app.convert_euid)(owner),
            relation: "owner".to_string(),
            object: (self.app.convert_euid)(owned),
        }
    }

    /// create the OpenFGA tuple representing that `child`'s parent folder is `parent`
    fn parent_folder(&self, child: &EntityUID, parent: &EntityUID) -> OpenFgaTuple {
        assert_eq!(parent.entity_type(), &self.gdtypes.folder);
        OpenFgaTuple {
            user: (self.app.convert_euid)(parent),
            relation: "parent".to_string(), // read this as "is the parent of"
            object: (self.app.convert_euid)(child),
        }
    }

    /// create the OpenFGA tuple representing that `object` is publicly viewable
    fn public_view(&self, object: &EntityUID) -> OpenFgaTuple {
        OpenFgaTuple {
            user: "user:*".to_string(),
            relation: "viewer".to_string(),
            object: (self.app.convert_euid)(object),
        }
    }

    /// convert a View into the corresponding OpenFGA tuple
    ///
    /// `view` should have type `View`.
    /// `object` is what the view rights are on.
    fn view_to_tuple(&self, view: &EntityUID, object: &EntityUID) -> OpenFgaTuple {
        assert_eq!(view.entity_type(), &self.gdtypes.view);
        let user_or_group = {
            let eid: &str = view.eid().as_ref();
            if let Some(user) = eid.strip_prefix("User ") {
                let user_euid = EntityUID::with_eid_and_type("User", user).unwrap();
                (self.app.convert_euid)(&user_euid)
            } else if let Some(group) = eid.strip_prefix("Group ") {
                let group_euid = EntityUID::with_eid_and_type("Group", group).unwrap();
                let mut s = (self.app.convert_euid)(&group_euid);
                s.push_str("#member");
                s
            } else {
                panic!("Expected all View eids to match one of these patterns")
            }
        };
        OpenFgaTuple {
            user: user_or_group,
            relation: "viewer".to_string(),
            object: (self.app.convert_euid)(object),
        }
    }
}

struct TinyTodoTypes {
    user: EntityType,
    team: EntityType,
    list: EntityType,
    app: EntityType,
}

impl TinyTodoTypes {
    fn new() -> Self {
        Self {
            user: utils::entity_type("User"),
            team: utils::entity_type("Team"),
            list: utils::entity_type("List"),
            app: utils::entity_type("Application"),
        }
    }
}

/// Converts Cedar entity data into `OpenFgaTuple`s, for both the Github encodings
struct TinyTodoConverter<'a> {
    app: &'a ExampleApp,
    types: TinyTodoTypes,
}

impl<'a> TinyTodoConverter<'a> {
    fn new(app: &'a ExampleApp) -> Self {
        Self {
            app,
            types: TinyTodoTypes::new(),
        }
    }

    /// Convert these Cedar entities into `OpenFgaTuple`s.
    /// The number of tuples will not necessarily be the same as the number of entities.
    fn convert_entities<'b>(
        &self,
        entities: impl IntoIterator<Item = &'b Entity>,
    ) -> Vec<OpenFgaTuple> {
        entities
            .into_iter()
            .flat_map(move |entity| {
                if entity.uid().entity_type() == &self.types.app {
                    // all Users are in all Applications
                    vec![OpenFgaTuple {
                        user: "user:*".to_string(),
                        relation: "member".to_string(),
                        object: (self.app.convert_euid)(&entity.uid()),
                    }]
                } else if entity.uid().entity_type() == &self.types.user {
                    entity
                        .ancestors()
                        .filter_map(|ancestor| self.parent_to_tuple(&entity.uid(), ancestor))
                        .collect()
                } else if entity.uid().entity_type() == &self.types.team {
                    entity
                        .ancestors()
                        .filter_map(|ancestor| self.parent_to_tuple(&entity.uid(), ancestor))
                        .collect()
                } else if entity.uid().entity_type() == &self.types.list {
                    let owner = utils::pv_expect_euid(
                        entity.get("owner").expect("List entity should have .owner"),
                    );
                    let owner_tuple = {
                        OpenFgaTuple {
                            user: (self.app.convert_euid)(&owner),
                            relation: "owner".to_string(), // read this as "is the owner of"
                            object: (self.app.convert_euid)(&entity.uid()),
                        }
                    };
                    let reader_tuple = {
                        let reader_team = utils::pv_expect_euid(
                            entity
                                .get("readers")
                                .expect("List entity should have .readers"),
                        );
                        self.reader_editor_to_tuple(&reader_team, &entity.uid(), false)
                    };
                    let editor_tuple = {
                        let editor_team = utils::pv_expect_euid(
                            entity
                                .get("editors")
                                .expect("List entity should have .editors"),
                        );
                        self.reader_editor_to_tuple(&editor_team, &entity.uid(), true)
                    };
                    vec![owner_tuple, reader_tuple, editor_tuple]
                } else {
                    vec![]
                }
            })
            .collect()
    }

    /// create the OpenFGA tuple representing membership of `child` in `parent` in
    /// the Cedar entity data
    fn parent_to_tuple(&self, child: &EntityUID, parent: &EntityUID) -> Option<OpenFgaTuple> {
        if parent.entity_type() == &self.types.team {
            // this is OK/expected
        } else if parent.entity_type() == &self.types.app {
            // we ignore edges to Application
            return None;
        } else {
            panic!("expected parent to be Team or Application")
        }
        let mut user = (self.app.convert_euid)(child);
        if child.entity_type() == &self.types.user {
            // nothing to do
        } else if child.entity_type() == &self.types.team {
            user.push_str("#member");
        } else {
            panic!("expected child to be User or Team");
        }
        Some(OpenFgaTuple {
            user,
            relation: "member".to_string(),
            object: (self.app.convert_euid)(parent),
        })
    }

    /// create the OpenFGA tuple representing that `grantee` can read/edit `list`
    ///
    /// if `editor` is true then it's an editor permission, else it's a reader permission
    fn reader_editor_to_tuple(
        &self,
        grantee: &EntityUID,
        list: &EntityUID,
        editor: bool,
    ) -> OpenFgaTuple {
        assert_eq!(list.entity_type(), &self.types.list);
        let mut user = (self.app.convert_euid)(grantee);
        if grantee.entity_type() == &self.types.user {
            // nothing to do
        } else if grantee.entity_type() == &self.types.team {
            user.push_str("#member");
        } else {
            panic!("expected reader/editor to be either a User or Team");
        }
        OpenFgaTuple {
            user,
            relation: if editor {
                "editor".to_string()
            } else {
                "reader".to_string()
            },
            object: (self.app.convert_euid)(list),
        }
    }
}

struct OpenFgaServer {
    /// Handle to the child process running the server
    child: Child,
}

impl OpenFgaServer {
    fn new() -> std::io::Result<Self> {
        let server = Self {
            child: Command::new(OPENFGA_BIN_PATH)
                .arg("run")
                .arg("--playground-enabled=false")
                .arg("--log-level=warn")
                .spawn()?,
        };
        std::thread::sleep(Duration::from_millis(20)); // give time for OpenFGA to start up. Experimentally, on my machine, 10 ms is insufficient and sometimes gives HTTP errors
        Ok(server)
    }
}

impl Drop for OpenFgaServer {
    fn drop(&mut self) {
        assert!(Command::new("kill")
            .args(["-s", "TERM", &self.child.id().to_string()])
            .status()
            .expect("failed to execute kill")
            .success());
    }
}
