use core::fmt;
use krabmaga::engine::agent::Agent;
use krabmaga::engine::fields::field_2d::{toroidal_distance, toroidal_transform, Location2D};
use krabmaga::engine::location::Real2D;
use krabmaga::engine::state::State;
use krabmaga::{rand, Uniform};
use krabmaga::rand::Rng;
use std::hash::{Hash, Hasher};
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;

use crate::model::state::Flocker;
use crate::{AVOIDANCE, COHESION, CONSISTENCY, JUMP, MOMENTUM, RANDOMNESS, SEED};
use krabmaga::Distribution;

#[derive(Clone, Copy)]
pub struct Bird {
    pub id: u32,
    pub loc: Real2D,
    pub last_d: Real2D,
}

impl Bird {
    pub fn new(id: u32, loc: Real2D, last_d: Real2D) -> Self {
        Bird { id, loc, last_d }
    }
}

impl Agent for Bird {
    fn step(&mut self, state: &mut dyn State) {
        let state = state.as_any().downcast_ref::<Flocker>().unwrap();
        let vec = state
            .field1
            .get_neighbors_within_relax_distance(self.loc, 10.0);

        let width = state.dim.0;
        let height = state.dim.1;

        let mut avoidance = Real2D { x: 0.0, y: 0.0 };

        let mut cohesion = Real2D { x: 0.0, y: 0.0 };
        let mut randomness = Real2D { x: 0.0, y: 0.0 };
        let mut consistency = Real2D { x: 0.0, y: 0.0 };

        if !vec.is_empty() {
            //avoidance
            let mut x_avoid = 0.0;
            let mut y_avoid = 0.0;
            let mut x_cohe = 0.0;
            let mut y_cohe = 0.0;
            let mut x_cons = 0.0;
            let mut y_cons = 0.0;
            let mut count = 0;

            for elem in &vec {
                if self.id != elem.id {
                    let dx = toroidal_distance(self.loc.x, elem.loc.x, width);
                    let dy = toroidal_distance(self.loc.y, elem.loc.y, height);
                    count += 1;

                    //avoidance calculation
                    let square = dx * dx + dy * dy;
                    x_avoid += dx / (square * square + 1.0);
                    y_avoid += dy / (square * square + 1.0);

                    //cohesion calculation
                    x_cohe += dx;
                    y_cohe += dy;

                    //consistency calculation
                    x_cons += elem.last_d.x;
                    y_cons += elem.last_d.y;
                }
            }

            if count > 0 {
                x_avoid /= count as f32;
                y_avoid /= count as f32;
                x_cohe /= count as f32;
                y_cohe /= count as f32;
                x_cons /= count as f32;
                y_cons /= count as f32;

                consistency = Real2D {
                    x: x_cons / count as f32,
                    y: y_cons / count as f32,
                };
            } else {
                consistency = Real2D {
                    x: x_cons,
                    y: y_cons,
                };
            }

            avoidance = Real2D {
                x: 400.0 * x_avoid,
                y: 400.0 * y_avoid,
            };

            cohesion = Real2D {
                x: -x_cohe / 10.0,
                y: -y_cohe / 10.0,
            };

            //randomness - uses the same rng of the ecs-experiment branch
            let mut rng = ChaCha8Rng::seed_from_u64(SEED);
            rng.set_stream(self.id as u64 + state.step);
            let mut range = Uniform::new(0.0f32, 1.0);
            let r1: f32 = range.sample(&mut rng);
            let x_rand = r1 * 2. - 1.;
            let r2: f32 = range.sample(&mut rng);
            let y_rand = r2 * 2. - 1.;
            // let mut rng = rand::thread_rng();
            // let r1: f32 = rng.gen();
            // let x_rand = r1 * 2.0 - 1.0;
            // let r2: f32 = rng.gen();
            // let y_rand = r2 * 2.0 - 1.0;

            let square = (x_rand * x_rand + y_rand * y_rand).sqrt();
            randomness = Real2D {
                x: 0.05 * x_rand / square,
                y: 0.05 * y_rand / square,
            };
        }

        let mom = self.last_d;

        let mut dx = COHESION * cohesion.x
            + AVOIDANCE * avoidance.x
            + CONSISTENCY * consistency.x
            + RANDOMNESS * randomness.x
            + MOMENTUM * mom.x;
        let mut dy = COHESION * cohesion.y
            + AVOIDANCE * avoidance.y
            + CONSISTENCY * consistency.y
            + RANDOMNESS * randomness.y
            + MOMENTUM * mom.y;

        let dis = (dx * dx + dy * dy).sqrt();
        if dis > 0.0 {
            dx = dx / dis * JUMP;
            dy = dy / dis * JUMP;
        }

        self.last_d = Real2D { x: dx, y: dy };

        let loc_x = toroidal_transform(self.loc.x + dx, width);
        let loc_y = toroidal_transform(self.loc.y + dy, height);

        self.loc = Real2D { x: loc_x, y: loc_y };
        if state.step == 200 {
            println!("bird {} step {} - cohesion {}, avoidance {}, consistency {}, randomness {}, mom {}, loc_x {} loc_y {}", self.id, state.step, cohesion, avoidance, consistency, randomness, mom, loc_x, loc_y);
        }
        drop(vec);
        state
            .field1
            .set_object_location(*self, Real2D { x: loc_x, y: loc_y });
    }
}

impl Hash for Bird {
    fn hash<H>(&self, state: &mut H)
        where
            H: Hasher,
    {
        self.id.hash(state);
    }
}

impl Eq for Bird {}

impl PartialEq for Bird {
    fn eq(&self, other: &Bird) -> bool {
        self.id == other.id
    }
}

impl Location2D<Real2D> for Bird {
    fn get_location(self) -> Real2D {
        self.loc
    }

    fn set_location(&mut self, loc: Real2D) {
        self.loc = loc;
    }
}

impl fmt::Display for Bird {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} loc {}", self.id, self.loc)
    }
}
