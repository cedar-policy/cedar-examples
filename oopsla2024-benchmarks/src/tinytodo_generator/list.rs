use std::collections::{BTreeMap, HashMap, HashSet};

use arbitrary::{Arbitrary, Unstructured};
use cedar_policy_core::ast::{EntityUID, PartialValue, Value};
use serde::Serialize;
use smol_str::SmolStr;

use super::common::{application, make_uid, Entity, HasId, NameGenerator, Uid};
use super::constants::NUM_LISTS;
use crate::utils;

#[derive(Debug, Clone, Serialize)]
pub struct List {
    euid: Uid,
    pub owner: Uid,
    name: String,
    tasks: Vec<Task>,
    pub readers: Uid,
    pub editors: Uid,
}

impl List {
    pub fn to_cedar_entity(&self) -> cedar_policy_core::ast::Entity {
        let mut attrs = HashMap::new();
        attrs.insert(
            "owner".into(),
            PartialValue::Value(self.owner.to_euid().into()),
        );
        attrs.insert("name".into(), PartialValue::Value(self.name.clone().into()));
        attrs.insert(
            "readers".into(),
            PartialValue::Value(self.readers.to_euid().into()),
        );
        attrs.insert(
            "editors".into(),
            PartialValue::Value(self.editors.to_euid().into()),
        );
        attrs.insert(
            "tasks".into(),
            PartialValue::Value(Value::set(
                self.tasks.iter().map(|task| task.to_value()),
                None,
            )),
        );
        cedar_policy_core::ast::Entity::new_with_attr_partial_value(
            self.euid.to_euid(),
            attrs,
            HashSet::new(),
            HashSet::new(),
            BTreeMap::new(),
        )
    }

    pub fn from_cedar_entity(e: cedar_policy_core::ast::Entity) -> Self {
        assert_eq!(e.uid().entity_type().to_string(), "List");
        Self {
            euid: e.uid().into(),
            owner: EntityUID::clone(&utils::pv_expect_euid(
                e.get("owner").expect("List entity should have .owner"),
            ))
            .into(),
            name: utils::pv_expect_string(e.get("name").expect("List entity should have .name"))
                .to_string(),
            tasks: utils::pv_expect_set(e.get("tasks").expect("List entity should have .tasks"))
                .map(Task::from_cedar_record)
                .collect(),
            readers: EntityUID::clone(&utils::pv_expect_euid(
                e.get("readers").expect("List entity should have .readers"),
            ))
            .into(),
            editors: EntityUID::clone(&utils::pv_expect_euid(
                e.get("editors").expect("List entity should have .editors"),
            ))
            .into(),
        }
    }
}

impl HasId for List {
    fn id(&self) -> Uid {
        self.euid.clone()
    }
}

#[derive(Debug, Clone, Serialize)]
struct Task {
    id: i64,
    name: SmolStr,
    state: TaskState,
}

impl Task {
    pub fn to_value(&self) -> Value {
        let attrs: [(SmolStr, _); 3] = [
            ("id".into(), Value::from(self.id)),
            ("name".into(), Value::from(self.name.clone())),
            ("state".into(), Value::from(format!("{:?}", self.state))),
        ];
        Value::record(attrs, None)
    }

    pub fn from_cedar_record(rec: &Value) -> Self {
        let map = utils::expect_record(rec);
        Self {
            id: utils::expect_int(map.get("id").expect("Task record should have .id")),
            name: utils::expect_string(map.get("name").expect("Task record should have .name")),
            state: match utils::expect_string(
                map.get("state").expect("Task record should have .state"),
            )
            .as_str()
            {
                "Checked" => TaskState::Checked,
                "Unchecked" => TaskState::Unchecked,
                s => panic!("unexpected task state: `{s}`"),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Arbitrary)]
enum TaskState {
    Checked,
    Unchecked,
}

pub fn arbitrary_lists<'b>(
    u: &'b mut Unstructured<'_>,
    users: &'b [Uid],
) -> arbitrary::Result<(Vec<List>, Vec<Entity>)> {
    NUM_LISTS.with(|num_lists| ListGenerator::new(u, users, *num_lists.borrow()).generate_lists())
}
struct ListGenerator<'a, 'b> {
    u: &'b mut Unstructured<'a>,
    users: &'b [Uid],
    count: usize,
    gen: NameGenerator,
}

impl<'a, 'b> ListGenerator<'a, 'b> {
    pub fn new(u: &'b mut Unstructured<'a>, users: &'b [Uid], count: usize) -> Self {
        Self {
            u,
            users,
            count,
            gen: NameGenerator::default(),
        }
    }

    fn generate_lists(&mut self) -> arbitrary::Result<(Vec<List>, Vec<Entity>)> {
        let mut lists = Vec::with_capacity(self.count);
        let mut entities = vec![];
        lists.reserve(self.count * 2);
        for _ in 0..self.count {
            let (l, r, w) = self.generate_list()?;
            lists.push(l);
            entities.push(r);
            entities.push(w);
        }

        Ok((lists, entities))
    }

    fn generate_list(&mut self) -> arbitrary::Result<(List, Entity, Entity)> {
        let id = self.gen.fresh(self.u)?;
        let owner = self.arbitrary_owner()?.clone();
        let name: String = self.u.arbitrary()?;
        let Tasks(tasks) = self.u.arbitrary()?;
        let readers = make_uid("Team", &format!("{id}_readers"));
        let writers = make_uid("Team", &format!("{id}_writers"));
        let uid = make_uid("List", &id);

        let list = List {
            euid: uid,
            owner,
            name,
            tasks,
            readers: readers.clone(),
            editors: writers.clone(),
        };

        Ok((list, make_orphan(readers), make_orphan(writers)))
    }

    fn arbitrary_owner(&mut self) -> arbitrary::Result<&Uid> {
        self.u.choose(self.users)
    }
}

fn make_orphan(uid: Uid) -> Entity {
    Entity {
        euid: uid,
        parents: vec![application()],
    }
}

struct Tasks(Vec<Task>);

impl<'a> Arbitrary<'a> for Tasks {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let count = u.int_in_range(0..=50)?;
        (0..count)
            .map(|id| arbitrary_task(u, id))
            .collect::<Result<Vec<_>, _>>()
            .map(Self)
    }
}

fn arbitrary_task(u: &mut Unstructured<'_>, id: i64) -> arbitrary::Result<Task> {
    let state = u.arbitrary()?;
    let name = u.arbitrary()?;
    Ok(Task { id, name, state })
}
