use rand::Rng;
use serde::{Deserialize, Serialize};

use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Read,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Champion {
    id: String,
    tier: u8,
    traits: Vec<i8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Trait {
    name: String,
    min: i8,
}

#[derive(Debug)]
pub struct Game {
    pub pool: HashMap<i8, Champion>,
    pub traits: HashMap<i8, Trait>,
    pub high_tier: bool,
    pub augment: String,
    pub tier_coefficient: f64,
}

impl Game {
    pub fn new() -> Self {
        Self {
            pool: HashMap::new(),
            traits: HashMap::new(),
            high_tier: false,
            augment: "standUnited".to_owned(),
            tier_coefficient: 1.0,
        }
    }

    pub fn read_json(
        &mut self,
        champ_json_name: &str,
        trait_json_name: &str,
        high_tier: bool,
        augment: &String,
        tier_coefficient: f64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::open(champ_json_name)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        self.pool = serde_json::from_str(&contents)?;

        contents.clear();
        file = File::open(trait_json_name)?;
        file.read_to_string(&mut contents)?;

        self.traits = serde_json::from_str(&contents)?;
        self.augment = augment.clone();
        self.high_tier = high_tier;
        self.tier_coefficient = tier_coefficient;

        Ok(())
    }

    pub fn get_traits(&self, solution: &Vec<i8>) -> HashMap<i8, i8> {
        let mut traits_map: HashMap<i8, i8> = HashMap::new();
        let mut champs_id_set: HashSet<i8> = HashSet::new();
        solution.iter().for_each(|s| {
            if !champs_id_set.contains(&s) {
                champs_id_set.insert(*s);
                self.pool[s].traits.iter().for_each(|&t| {
                    *traits_map.entry(t).or_insert(0) += 1;
                })
            }
        });

        traits_map
    }

    pub fn evaluate(&self, solution: &Vec<i8>) -> (i32, i32) {
        let mut traits_map: HashMap<i8, i8> = HashMap::new();
        let mut champs_id_set: HashSet<i8> = HashSet::new();
        let mut sum_tier = 0;
        let mut eval = 0;
        solution.iter().for_each(|s| {
            let champ = &self.pool[s];
            if !champs_id_set.contains(&s) {
                champs_id_set.insert(*s);
                champ.traits.iter().for_each(|&t| {
                    *traits_map.entry(t).or_insert(0) += 1;
                })
            }
            sum_tier += champ.tier;
        });
        traits_map.into_iter().for_each(|(id, num)| {
            if self.traits[&id].min <= num {
                eval += 1;
            }
        });

        (
            eval,
            eval + if self.high_tier {
                ((sum_tier as f64 / 5.0) * self.tier_coefficient) as i32
            } else {
                0
            },
        )
    }

    pub fn local_search(&self, init: &Vec<i8>) -> Vec<i8> {
        let mut solution = init.clone();
        let mut better_option: Vec<i8> = vec![];
        let mut eval_first = self.evaluate(&solution).1;
        let mut eval_temp: i32;
        let mut eval_better_option = 0;
        let size = solution.len();
        let pool_size = self.pool.len();
        loop {
            for i in 0..size {
                for j in 1..=pool_size {
                    let prev_i = solution[i];
                    solution[i] = j as i8;
                    eval_temp = self.evaluate(&solution).1;
                    if eval_temp > eval_better_option {
                        better_option = solution.clone();
                        eval_better_option = eval_temp;
                    }
                    solution[i] = prev_i;
                }
            }
            if eval_better_option > eval_first {
                solution = better_option;
                better_option = vec![];
                eval_first = eval_better_option;
                eval_better_option = 0;
            } else {
                return solution;
            }
        }
    }

    pub fn ils(&self, init: &Vec<i8>) -> Vec<i8> {
        let mut rng = rand::thread_rng();
        let n_iter = 500;
        let mut i: usize;
        let mut j: usize;
        let len = init.len();
        let pert_strength = if init.len() < 3 {init.len()} else {3};
        let mut solution = init.clone();
        let mut eval_solution: i32;
        let mut best_solution = solution.clone();
        let mut eval_best_solution = self.evaluate(&mut best_solution).1;

        for _ in 0..n_iter {
            solution = self.local_search(&solution);
            eval_solution = self.evaluate(&mut solution).1;

            if eval_best_solution < eval_solution {
                best_solution = solution.clone();
                eval_best_solution = eval_solution;
            }

            for _ in 0..pert_strength {
                i = rng.gen_range(0..len);
                j = rng.gen_range(1..=self.pool.len());

                solution[i] = j as i8;
            }
        }
        best_solution
    }

    pub fn evaluate_bd(&self, solution: &Vec<i8>) -> (i32, i32) {
        let mut traits_map: HashMap<i8, i8> = HashMap::new();
        let mut champs_id_set: HashSet<i8> = HashSet::new();
        let mut eval = 0;
        let mut sum_tier = 0;
        solution.iter().for_each(|s| {
            let champ = &self.pool[s];
            if !champs_id_set.contains(&s) {
                champs_id_set.insert(*s);
                champ.traits.iter().for_each(|&t| {
                    *traits_map.entry(t).or_insert(0) += 1;
                })
            } else {
                eval += 1;
            }
            sum_tier += champ.tier;
        });
        traits_map.into_iter().for_each(|(id, num)| {
            if self.traits[&id].min <= num {
                eval += 1;
            }
        });

        if eval > 0 {
            return (eval, eval * 2);
        }
        (
            eval,
            eval - if self.high_tier {
                (sum_tier as f64/10.0 * self.tier_coefficient) as i32
            } else {
                0
            },
        )
    }

    pub fn local_search_bd(&self, init: &Vec<i8>) -> Vec<i8> {
        let mut solution = init.clone();
        let mut better_option: Vec<i8> = vec![];
        let mut eval_first = self.evaluate_bd(&solution).1;
        let mut eval_temp: i32;
        let mut eval_better_option = i32::MAX;
        let size = solution.len();
        let pool_size = self.pool.len();
        loop {
            for i in 0..size {
                for j in 1..=pool_size {
                    let prev_i = solution[i];
                    solution[i] = j as i8;
                    eval_temp = self.evaluate_bd(&solution).1;
                    if eval_temp < eval_better_option {
                        better_option = solution.clone();
                        eval_better_option = eval_temp;
                    }
                    solution[i] = prev_i;
                }
            }
            if eval_better_option < eval_first {
                solution = better_option;
                better_option = vec![];
                eval_first = eval_better_option;
                eval_better_option = i32::MAX;
            } else {
                return solution;
            }
        }
    }

    pub fn ils_bd(&self, init: &Vec<i8>) -> Vec<i8> {
        let mut rng = rand::thread_rng();
        let n_iter = 500;
        let mut i: usize;
        let mut j: usize;
        let len = init.len();
        let pert_strength = if init.len() < 3 {init.len()} else {3};
        let mut solution = init.clone();
        let mut eval_solution: i32;
        let mut best_solution = solution.clone();
        let mut eval_best_solution = self.evaluate_bd(&best_solution).1;

        for _ in 0..n_iter {
            solution = self.local_search_bd(&solution);
            eval_solution = self.evaluate_bd(&solution).1;

            if eval_best_solution > eval_solution {
                best_solution = solution.clone();
                eval_best_solution = eval_solution;
            }

            for _ in 0..pert_strength {
                i = rng.gen_range(0..len);
                j = rng.gen_range(1..=self.pool.len());

                solution[i] = j as i8;
            }
        }
        best_solution
    }
}
