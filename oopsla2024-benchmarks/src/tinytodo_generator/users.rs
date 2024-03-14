use arbitrary::{Result, Unstructured};

use super::common::{make_uid, Entity, NameGenerator, Uid};

pub fn arbitrary_users<'b>(
    u: &'b mut Unstructured<'_>,
    teams: &'b [Uid],
    count: usize,
) -> arbitrary::Result<Vec<Entity>> {
    UserGenerator::new(u, teams).arbitrary_users(count)
}

struct UserGenerator<'a, 'b> {
    u: &'b mut Unstructured<'a>,
    teams: &'b [Uid],
    gen: NameGenerator,
}

impl<'a, 'b> UserGenerator<'a, 'b> {
    fn new(u: &'b mut Unstructured<'a>, teams: &'b [Uid]) -> Self {
        Self {
            u,
            teams,
            gen: NameGenerator::default(),
        }
    }

    fn arbitrary_users(&mut self, count: usize) -> Result<Vec<Entity>> {
        let mut users = Vec::with_capacity(count);
        for _ in 0..count {
            users.push(self.arbitrary_user()?);
        }
        Ok(users)
    }

    fn arbitrary_user(&mut self) -> Result<Entity> {
        let id = self.gen.fresh(self.u)?;
        let num_parents = self.u.int_in_range(0..=3)?;
        let mut parents = vec![];
        for _ in 0..num_parents {
            parents.push((*self.u.choose(self.teams)?).clone());
        }
        let uid = make_uid("User", &id);
        Ok(Entity { euid: uid, parents })
    }
}
