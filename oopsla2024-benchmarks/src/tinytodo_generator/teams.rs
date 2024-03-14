use std::cell::RefCell;

use arbitrary::{Result, Unstructured};

use super::common::{make_uid, Entity, NameGenerator, Uid};

pub fn generate_teams(u: &mut Unstructured<'_>, teams: usize) -> Result<Vec<Entity>> {
    let bound = (teams as f64).log2().ceil() as usize;
    let (_, teams) = TeamGenerator::new(u).generate_tree(bound)?;
    Ok(tree_to_entities(teams, None))
}

struct TeamGenerator<'a, 'b> {
    u: &'b mut Unstructured<'a>,
    gen: NameGenerator,
}

impl<'a, 'b> TeamGenerator<'a, 'b> {
    pub fn new(u: &'b mut Unstructured<'a>) -> Self {
        Self {
            u,
            gen: NameGenerator::default(),
        }
    }

    fn generate_tree(&mut self, bound: usize) -> Result<(usize, TeamTree)> {
        if bound == 0 {
            self.generate_leaf()
        } else {
            match self.u.int_in_range(0..=4)? {
                4 => self.generate_leaf(),
                _ => self.generate_branch(bound - 1),
            }
        }
    }

    fn generate_branch(&mut self, bound: usize) -> Result<(usize, TeamTree)> {
        let id = self.gen.fresh(self.u)?;
        let team = make_uid("Team", &id);
        let (left_size, left) = self.generate_tree(bound)?;
        let (right_size, right) = self.generate_tree(bound)?;
        let size = 1 + left_size + right_size;
        Ok((size, TeamTree::node(team, left, right)))
    }

    fn generate_leaf(&mut self) -> Result<(usize, TeamTree)> {
        let id = self.gen.fresh(self.u)?;
        let team = make_uid("Team", &id);
        Ok((1, TeamTree::Leaf(Some(team))))
    }
}

fn tree_to_entities(t: TeamTree, parent: Option<Uid>) -> Vec<Entity> {
    match t {
        TeamTree::Node { team, left, right } => {
            let this = Entity {
                euid: team.clone(),
                parents: option_to_vec(parent),
            };
            let mut left = tree_to_entities(left.into_inner(), Some(team.clone()));
            let mut right = tree_to_entities(right.into_inner(), Some(team));
            left.append(&mut right);
            left.push(this);
            left
        }
        TeamTree::Leaf(Some(euid)) => vec![Entity {
            euid,
            parents: option_to_vec(parent),
        }],
        TeamTree::Leaf(None) => vec![],
    }
}

fn option_to_vec<T>(o: Option<T>) -> Vec<T> {
    match o {
        Some(t) => vec![t],
        None => vec![],
    }
}

enum TeamTree {
    Node {
        team: Uid,
        left: Box<RefCell<TeamTree>>,
        right: Box<RefCell<TeamTree>>,
    },
    Leaf(Option<Uid>),
}

impl TeamTree {
    pub fn node(team: Uid, left: TeamTree, right: TeamTree) -> Self {
        Self::Node {
            team,
            left: Box::new(RefCell::new(left)),
            right: Box::new(RefCell::new(right)),
        }
    }
}
