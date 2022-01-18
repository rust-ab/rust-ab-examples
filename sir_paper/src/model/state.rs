use crate::model::node::{NetNode, NodeStatus};
use crate::{STEP, DISCRETIZATION, WIDTH, HEIGHT, NUM_NODES, TOROIDAL, INIT_EDGES, INITIAL_INFECTED};
use rust_ab::engine::fields::network::Network;
use rust_ab::engine::fields::{field::Field, field_2d::Field2D};
use rust_ab::engine::schedule::Schedule;
use rust_ab::engine::state::State;
use rust_ab::engine::location::Real2D;
use rust_ab::rand::Rng;
use rust_ab::rand;
use std::any::Any;
use std::{
    sync::{Arc, Mutex}
};

pub struct EpidemicNetworkState {
    pub step: u64,
    pub field1: Field2D<NetNode>,
    pub network: Network<NetNode, String>,
    pub infected_nodes: Arc<Mutex<Vec<u32>>> // each position of the array corresponds to one node
}

impl EpidemicNetworkState {
    pub fn new() -> EpidemicNetworkState {
        EpidemicNetworkState {
            step: 0,
            field1: Field2D::new(WIDTH, HEIGHT, DISCRETIZATION, TOROIDAL),
            network: Network::new(false),
            infected_nodes: Arc::new(Mutex::new(vec![0; NUM_NODES as usize])) // dimension is NUM_NODE
        }
    }

    // GA required new function
    // pub fn new_with_parameters(parameters: &String) -> EpidemicNetworkState{
    //     let mut positions: Vec<u32> = Vec::new();

    //     for i in parameters.chars() {
    //         positions.push(i.to_digit(10).unwrap());
    //     }

    //     EpidemicNetworkState::new(positions)
    // }
}


impl State for EpidemicNetworkState {
    fn init(&mut self, schedule: &mut Schedule) {
        let mut rng = rand::thread_rng();
        let my_seed: u64 = 0;
        let mut node_set = Vec::new();
        self.network = Network::new(false);

        let mut infected_counter = 0;
        // initial percentage of infected node 
        // generates casual nodes
        for node_id in 0..NUM_NODES {
            let mut init_status: NodeStatus = NodeStatus::Susceptible;

            // generated exactly INITIAL_INFECTED * NUM_NODES infected nodes
            if infected_counter != (INITIAL_INFECTED * NUM_NODES as f32) as u32 {
                init_status = NodeStatus::Infected;
                infected_counter += 1;
            }
           
            let r1: f32 = rng.gen();
            let r2: f32 = rng.gen();

            let node = NetNode::new(
                node_id,
                Real2D {
                    x: WIDTH * r1,
                    y: HEIGHT * r2,
                },
                init_status,
            );
           
            self.field1.set_object_location(node, node.loc);
            self.network.add_node(node);
            schedule.schedule_repeating(Box::new(node), 0.0, 0);
            node_set.push(node);
        }
        self.network.preferential_attachment_BA_with_seed(&node_set, INIT_EDGES, my_seed);
    }

    fn update(&mut self, step: u64) {
        self.field1.lazy_update();
        self.network.update();
        self.step = step;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_state_mut(&mut self) -> &mut dyn State {
        self
    }

    fn as_state(&self) -> &dyn State {
        self
    }

    // fn before_step(&mut self, schedule: &mut Schedule) {
    //     let mut susceptible: usize = 0;
    //     let mut infected: usize = 0;
    //     let mut resistant: usize = 0;
    //     let agents = schedule.get_all_events();

    //     for n in agents {
    //         let agent = n.downcast_ref::<NetNode>().unwrap();
    //         match agent.status {
    //             NodeStatus::Susceptible => {
    //                 susceptible += 1;
    //             }
    //             NodeStatus::Infected => {
    //                 infected += 1;
    //             }
    //             NodeStatus::Resistant => {
    //                 resistant += 1;
    //             }
    //         }
    //     }
    //     println!(
    //         "Susceptible: {:?} Infected: {:?} Resistant: {:?} Tot: {:?}",
    //         susceptible,
    //         infected,
    //         resistant,
    //         susceptible + infected + resistant
    //     );
    // }

    fn end_condition(&mut self, schedule: &mut Schedule) -> bool {
        println!("Running end condition");
        let mut infected: usize = 0;
        let agents = schedule.get_all_events();

        for n in agents {
            let agent = n.downcast_ref::<NetNode>().unwrap();
            if agent.status == NodeStatus::Infected {
                infected += 1;
            }
        }
        if self.step == 10 { // compute the R0 after 30 days
            let mut counter = 0;
            let mut value = 0;
            let infected_nodes = self.infected_nodes.lock().unwrap();
            for i in 0..infected_nodes.len(){
                if infected_nodes[i] != 0{
                    counter += 1;
                    value += infected_nodes[i]; 
                }
            }
            let rt: f32 = (value as f32 / counter as f32) as f32;
            println!("RT is {}", rt); 
        }
        if infected == 0 {
            // println!("No more infected nodes at step {}, exiting.", schedule.step);
            return true;
        }
        false
    }

}
