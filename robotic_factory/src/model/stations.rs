use std::fmt;
use std::hash::{Hash, Hasher};

use krabmaga::{rand, Rng};
use krabmaga::engine::agent::Agent;
use krabmaga::engine::fields::field_2d::Location2D;
use krabmaga::engine::location::Real2D;
use krabmaga::engine::state::State;

use crate::model::robot_factory::RobotFactory;
use crate::model::robot_factory::{Robot, RobotFactory};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum StationType {
    Sticher,
    Cutter,
    Finisher,
    LoadingDock,
    StorageRoom,
    RobotRoom,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct FinisherInformation {
    process_time: u32,
    progress: u32,
    is_delux: bool,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Station {
    id: u32,
    location: Real2D,
    material_management: MaterialManagement,
    station_type: StationType,

    finisher_information: FinisherInformation,
}

impl Station {
    pub fn new(id: u32, location: Real2D, station_type: StationType, mut is_delux_finisher: bool) -> Station {
        if station_type != StationType::Finisher {
            is_delux_finisher = false;
        }

        Station {
            id,
            location,
            material_management: MaterialManagement::default(),
            station_type,
            finisher_information: FinisherInformation {
                process_time: if is_delux_finisher { 7 } else { 4 },
                progress: 0,
                is_delux: is_delux_finisher,
            },
        }
    }

    pub fn get_station_type(&self) -> StationType {
        self.station_type
    }

    pub fn try_convert_one_supply(&mut self) {
        if self.material_management.has_supply() {
            self.material_management.decrement_supply();
            self.material_management.increment_products();
        }
    }
}

impl Location2D<Real2D> for Station {
    fn get_location(self) -> Real2D {
        self.location
    }

    fn set_location(&mut self, loc: Real2D) {
        self.location = loc;
    }
}

impl Hash for Station {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl fmt::Display for Station {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}({})", self.station_type, self.id)
    }
}

impl Agent for Station {
    fn step(&mut self, state: &mut dyn State) {
        let state_mut = state.as_any_mut().downcast_mut::<RobotFactory>().unwrap();

        match self.station_type {
            StationType::Sticher | StationType::Cutter => {
                //make-garments (except for finish call)
                self.try_convert_one_supply();
            }
            StationType::Finisher => {
                // finish
                if self.material_management.has_supply() {
                    self.finisher_information.progress += 1;
                    self.material_management.decrement_supply();
                }

                if self.finisher_information.progress >= self.finisher_information.process_time {
                    self.finisher_information.progress = 0;
                    self.material_management.increment_products();
                }
            }
            StationType::LoadingDock => {
                // deliver-more-material-sheets
                if rand::thread_rng().gen_bool(0.03) && self.material_management.get_products_count() < 3 {
                    self.material_management.add_products(rand::thread_rng().gen_range(0..10));
                }

                if rand::thread_rng().gen_bool(0.03) {
                    for _ in 0..rand::thread_rng().gen_range(0..3) {
                        state_mut.bump_required_orders(rand::thread_rng().gen_bool(0.5));
                    }
                }
            }
            StationType::StorageRoom => {}
            StationType::RobotRoom => {
                //recharge

                let robots = state_mut.get_robots();

                for mut robot in robots.iter() {
                    let robot_loc = robot.borrow().get_location();
                    let station_loc = self.get_location();

                    let distance = (robot_loc.x - station_loc.x).powi(2) + (robot_loc.y - station_loc.y).powi(2);

                    if distance <= 4.0 {
                        robot.borrow().charge(5, state_mut);
                    }
                }
            }
        }
    }
}

//----------------OrderManagement----------------
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct MaterialManagement {
    supply: u32,
    products: u32,
}

impl MaterialManagement {
    pub fn has_supply(&self) -> bool {
        self.supply > 0
    }
    pub fn get_supply_count(&self) -> u32 {
        self.supply
    }
    pub fn get_products_count(&self) -> u32 {
        self.products
    }
    pub fn has_products(&self) -> bool {
        self.products > 0
    }

    pub fn decrement_supply(&mut self) {
        self.supply -= 1;
    }
    pub fn increment_products(&mut self) {
        self.products += 1;
    }
    pub fn add_supply(&mut self, amount: u32) {
        self.supply += amount;
    }
    pub fn add_products(&mut self, amount: u32) {
        self.products += amount;
    }
}
