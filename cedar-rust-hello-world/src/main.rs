/*
 * Copyright 2022-2023 Amazon.com, Inc. or its affiliates. All Rights Reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#![forbid(unsafe_code)]
use cedar_policy::PrincipalConstraint::{Any, Eq, In};
use cedar_policy::{
    Authorizer, Context, Decision, Entities, Entity, EntityId, EntityTypeName, EntityUid, Policy,
    PolicyId, PolicySet, Request, Response, RestrictedExpression, Schema, SlotId, Template,
    ValidationMode, ValidationResult, Validator,
};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

fn main() {
    //policy parsing example
    parse_policy();

    //constructing context from JSON values
    json_context();

    // constructing entities from JSON
    entity_json();
    // constructing entities from Rust objects
    entity_objects();

    //validate a policy
    validate();

    //Getting policy annotations
    annotate();

    //print a policy in JSON format
    to_json();
}
/// parse a policy
fn parse_policy() {
    println!("Example: Parsing a Cedar Policy");
    // this policy has a type error, but parses.
    let src = r#"
    permit(
        principal == User::"bob",
        action == Action::"view",
        resource
    )
    when { 10 > "hello" };
"#;
    let p = PolicySet::from_str(src);
    match p {
        Ok(pset) => {
            let pid = PolicyId::from_str("policy_id_00").unwrap();
            let policy = PolicySet::policy(&pset, &pid);
            if let Some(p) = policy {
                println!("Policy:{}", p);
                let pr = Policy::principal_constraint(p);
                match pr {
                    Any => println!("No Principal"),
                    In(euid) => println!("Principal Constraint: Principal in {}", euid),
                    Eq(euid) => println!("Principal Constraint: Principal=={}", euid),
                }
            }
        }
        Err(e) => {
            println!("{:?}", e)
        }
    }
}

/// An example for constructing context from JSON value
fn json_context() {
    println!("json_context example");
    let v: serde_json::value::Value = serde_json::json!(
            {
                 "vendor":{
                   "groups":["finance"],
                    "ip_address":{"__extn":{"fn":"ip", "arg":"10.0.1.101"}}
                }
            }
    );

    let c = Context::from_json_value(v, None).unwrap();

    let (p, a, r) = create_p_a_r();
    // create a request
    let request: Request = Request::new(Some(p), Some(a), Some(r), c);

    // create a policy
    let s = r#"permit(
        principal == User::"alice",
        action == Action::"view",
        resource == Album::"trip"
      )when{
        context.vendor.ip_address.isIpv4()
      };
    "#;
    let ps = PolicySet::from_str(s).expect("policy error");

    let entities = create_entities_json();
    let ans = execute_query(&request, &ps, entities);

    print_response(ans);
}

/// An example for constructing entities from JSON
fn entity_json() {
    println!("Example: Constructing entities from JSON");

    let (p, a, r) = create_p_a_r();

    let t1: serde_json::value::Value = serde_json::json!(
        {
            "sub": "1232-1232",
            "groups": {
                "1234-1242": {
                    "group_id": "1234-1242",
                    "group_name": "test-group"
                }
            }
        }
    );
    let mut context2: HashMap<String, RestrictedExpression> = HashMap::new();

    context2.insert(
        "iam_idc".to_string(),
        RestrictedExpression::from_str(&t1.to_string()).unwrap(),
    );
    let c = Context::from_pairs(context2);

    let request: Request = Request::new(Some(p), Some(a), Some(r), c);

    // create a policy
    let s = "
    permit(
        principal == User::\"alice\",
        action == Action::\"view\",
        resource == Album::\"trip\"
      )when{
          principal.ip_addr.isIpv4() &&
          principal.age > 15 &&
          context.iam_idc.groups has \"1234-1242\" &&
          context.iam_idc.groups[\"1234-1242\"].group_id ==  \"1234-1242\" &&
          context.iam_idc.groups[\"1234-1242\"].group_name == \"test-group\"
      };
    ";

    let p = PolicySet::from_str(s).expect("policy error");

    let entities = create_entities_json();

    let ans = execute_query(&request, &p, entities);

    print_response(ans);
}

/// An example for constructing entities from Rust objects
fn entity_objects() {
    println!("Entity objects example");
    let p = EntityUid::from_type_name_and_id(
        EntityTypeName::from_str("User").unwrap(),
        EntityId::from_str("cat").unwrap(),
    );
    let a: EntityUid = EntityUid::from_type_name_and_id(
        EntityTypeName::from_str("Action").unwrap(),
        EntityId::from_str("view").unwrap(),
    );

    let r: EntityUid = EntityUid::from_type_name_and_id(
        EntityTypeName::from_str("Album").unwrap(),
        EntityId::from_str("summer").unwrap(),
    );

    let c = Context::empty();
    let request: Request = Request::new(Some(p), Some(a), Some(r), c);

    // create a policy
    let c1 = "permit(
         principal == User::\"alice\",
         action == Action::\"view\",
         resource == Album::\"trip\"
       );
     ";

    // create a policy template
    let c2 = "permit(
         principal == ?principal,
         action == Action::\"view\",
         resource == ?resource
       );
     ";

    // add an inline policy to the policy set
    let mut p = PolicySet::from_str(c1).expect("policy error");
    let t = Template::parse(Some("policy01".to_string()), c2).unwrap();
    let id1 = t.id().clone();
    // add a template to the policy set
    p.add_template(t).unwrap();

    // create a principal and a resource entities to instantiate the template
    let mut v = HashMap::new();
    let entity1 = EntityUid::from_type_name_and_id(
        EntityTypeName::from_str("User").unwrap(),
        EntityId::from_str("bob").unwrap(),
    );

    let entity2 = EntityUid::from_type_name_and_id(
        EntityTypeName::from_str("Album").unwrap(),
        EntityId::from_str("trip").unwrap(),
    );

    v.insert(SlotId::principal(), entity1);
    v.insert(SlotId::resource(), entity2);
    let id2 = PolicyId::from_str("id2").unwrap();

    // instantiate the template
    p.link(id1.clone(), id2, v).expect("Instantiation failed!");

    // create a principal and a resource entities to instantiate the template
    let mut v2 = HashMap::new();
    let entity3 = EntityUid::from_type_name_and_id(
        EntityTypeName::from_str("User").unwrap(),
        EntityId::from_str("cat").unwrap(),
    );

    let entity4 = EntityUid::from_type_name_and_id(
        EntityTypeName::from_str("Album").unwrap(),
        EntityId::from_str("summer").unwrap(),
    );

    v2.insert(SlotId::principal(), entity3);
    v2.insert(SlotId::resource(), entity4);
    let id3 = PolicyId::from_str("policy_id003").unwrap();

    // link the template (another template-linked policy)
    p.link(id1, id3, v2).expect("Linking failed!");

    let ans = execute_query(&request, &p, create_entities_obj());

    print_response(ans);
}

///  create entities from using Rust objects
/*
      Create a User entity
      UID: User::"alice"
      Attributes:
        age:17
        ip_addr:10.0.0.1
*/
fn create_entities_obj() -> Entities {
    let u = EntityUid::from_type_name_and_id(
        EntityTypeName::from_str("User").unwrap(),
        EntityId::from_str("alice").unwrap(),
    );

    let mut attrs = HashMap::new();
    let exp = RestrictedExpression::from_str("17").unwrap();
    attrs.insert("age".to_owned(), exp);
    attrs.insert(
        "ip_addr".to_owned(),
        RestrictedExpression::from_str("ip(\"10.0.0.1\")").unwrap(),
    );

    //println!("ATTRS:{:?}", attrs);

    let user = Entity::new(u, attrs, HashSet::new());
    let mut v = vec![user];

    // create an action entity
    let t = EntityUid::from_type_name_and_id(
        EntityTypeName::from_str("Action").unwrap(),
        EntityId::from_str("view").unwrap(),
    );

    let action = Entity::new(t, HashMap::new(), HashSet::new());

    v.push(action);

    //  create a resource entity
    let t = EntityUid::from_type_name_and_id(
        EntityTypeName::from_str("Album").unwrap(),
        EntityId::from_str("trip").unwrap(),
    );

    let resource = Entity::new(t, HashMap::new(), HashSet::new());
    v.push(resource);

    // create the Entities
    Entities::from_entities(v).unwrap()
}

///create entities from JSON
fn create_entities_json() -> Entities {
    let e = r#"[
        {
            "uid": {"type":"User","id":"alice"},
            "attrs": {
                "age":17,
                "ip_addr":{"__extn":{"fn":"ip", "arg":"10.0.1.101"}}
            },
            "parents": []
        },
        {
            "uid":{"type":"Action","id":"view"},
            "attrs": {},
            "parents": []
        },
        {
            "uid": {"__entity":{"type":"Album","id":"trip"}},
            "attrs": {},
            "parents": []
        }
    ]"#;

    Entities::from_json_str(e, None).expect("entity error")
}

/// Prints the Answer from the Authorization
fn print_response(ans: Response) {
    match ans.decision() {
        Decision::Allow => println!("ALLOW"),
        Decision::Deny => println!("DENY"),
    }

    println!();
    for err in ans.diagnostics().errors() {
        println!("{}", err);
    }

    println!();

    println!("note: this decision was due to the following policies:");
    for reason in ans.diagnostics().reason() {
        println!("  {}", reason);
    }
    println!();
}

/// This uses the waterford API to call the authorization engine.
fn execute_query(request: &Request, policies: &PolicySet, entities: Entities) -> Response {
    let authorizer = Authorizer::new();
    authorizer.is_authorized(request, &policies, &entities)
}

fn validate() {
    println!("Example: Validating a Policy");
    let src = r#"
    permit(
        principal == User::"bob",
        action == Action::"view",
        resource == Album::"trip"
    )
    when { 
        
        principal.age > 18

    };
"#;
    let sc = r#"
    {
        "": {
            "entityTypes": {
                "User": {
                    "shape": {
                        "type": "Record",
                        "attributes": {
                            "age": {
                                "type": "Long",
                                "name": "age"
                            }
                        }
                    },
                    "memberOfTypes": [
                        "UserGroup"
                    ]
                },
                
                "UserGroup": {
                    "memberOfTypes": []
                },
                
                "Album": {
                    "memberOfTypes": [
                        "Album"
                    ]
                }
            },
            "actions": {
                "view": {
                    "appliesTo": {
                        "resourceTypes": [
                            "Album"
                        ],
                        "principalTypes": [
                            "User"
                        ]
                    }
                }
            }
        }
    }
        "#;

    let p = PolicySet::from_str(src).unwrap();
    let schema = Schema::from_str(sc).unwrap();
    let validator = Validator::new(schema);

    let result = Validator::validate(&validator, &p, ValidationMode::default());
    if ValidationResult::validation_passed(&result) {
        println!("Validation Passed");
    } else {
        println!("Validation Failed.");
        let e = ValidationResult::validation_errors(&result);
        for err in e {
            println!("{}", err);
        }
    }
    println!();
}

fn annotate() {
    println!("Example: Policy Annotations");
    let src = r#"
    @advice("This policy allows alice to access the album")
    @type("simple")
    permit(
        principal == User::"alice",
        action == Action::"view",
        resource == Album::"trip"
    );
"#;

    let policies = PolicySet::from_str(src).unwrap();
    let (p, a, r) = create_p_a_r();
    let request: Request = Request::new(Some(p), Some(a), Some(r), Context::empty());
    let ans = execute_query(&request, &policies, Entities::empty());
    for reason in ans.diagnostics().reason() {
        //print all the annotations
        for (key, value) in policies.policy(&reason).unwrap().annotations() {
            println!("PolicyID: {}\tKey:{} \tValue:{}", reason, key, value);
        }
    }
    println!();
}

fn to_json() {
    println!("Example: Policy in JSON format");
    let src = r#"
    permit(
        principal == User::"bob",
        action == Action::"view",
        resource == Album::"trip"
    )
    when { 
        
        principal.age < 18

    };
    "#;
    let p = Policy::parse(None, src).unwrap();
    println!("{}", p);
    //convert the policy to JSON
    let json = p.to_json().unwrap();
    println!("{}", json);

    //create a policy from JSON
    let p2 = Policy::from_json(None, json).unwrap();

    println!("{}", p2);
}

fn create_p_a_r() -> (EntityUid, EntityUid, EntityUid) {
    let p_eid = EntityId::from_str("alice").unwrap(); // does not go through the parser
    let p_name: EntityTypeName = EntityTypeName::from_str("User").unwrap(); // through parse_name(s)
    let p = EntityUid::from_type_name_and_id(p_name, p_eid);

    let a_eid = EntityId::from_str("view").unwrap(); // does not go through the parser
    let a_name: EntityTypeName = EntityTypeName::from_str("Action").unwrap(); // through parse_name(s)
    let a = EntityUid::from_type_name_and_id(a_name, a_eid);

    let r_eid = EntityId::from_str("trip").unwrap(); // does not go through the parser
    let r_name: EntityTypeName = EntityTypeName::from_str("Album").unwrap(); // through parse_name(s)
    let r = EntityUid::from_type_name_and_id(r_name, r_eid);
    (p, a, r)
}
