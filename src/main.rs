use rand::prelude::*;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Gender {
	Male,
	Female
}

impl Gender {
	fn new() -> Self {
		let mut rng = rand::thread_rng();
		let genders = [Gender::Male, Gender::Female];
		
		return genders.choose(&mut rng).unwrap().clone();
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Individual {
	pub identity: String,
	pub gender: Gender,
	// a list of floats that represents how much does this person weight on different attributes
	pub preference_weights: Vec<f32>,
	// a list of integers that represents how much does this person score on each attribute
	pub ratings: Vec<f32>,
	// a list to record the individuals that rejected this individual
	pub blacklist: Vec<String>, 
	// a field that stores the previously accepted candidate
	pub candidate: Option<String>,
	// a field that stores the previously accepted candidate's score
	pub candidate_score: Option<f32>
}

impl std::fmt::Display for Individual {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "Identity: {}, {:#?}", self.identity, self.gender);
		writeln!(f, "Preference Weights: {:?}", self.preference_weights);
		writeln!(f, "Ratings: {:?}", self.ratings);
		writeln!(f, "Blacklist: {:?}", self.blacklist);
		writeln!(f, "Candidate: {:?}", self.candidate);
		writeln!(f, "Candidate Score: {:?}", self.candidate_score);
		
		return Ok(());
	}
}

impl Individual {
	/// use this method to generate an individual
	/// the preference complexity specifies the number of preference_weights
	/// and ratings will be used. 
	pub fn new(
		preference_complexity: i8, 
		specified_predefined_weights: Option<Vec<f32>>
	) -> Self {
		
		let mut rng = rand::thread_rng();
		
		let mut predefined_weights: Vec<f32> = Vec::new();	
		
		if specified_predefined_weights.is_some() {
			// if the `predefined_weights` is specified, use the specified the weights
			if specified_predefined_weights.clone().unwrap().len() != preference_complexity as usize {
				panic!("Wrong size of specified predefined weights!");
			} else {
				predefined_weights = specified_predefined_weights.unwrap();
			}
		} else {
			// generate random weights based on the given complexity
			// in case if the weights are not specified. 
			for _ in 0..preference_complexity {
				let weight: f32 = rng.r#gen();
				
				predefined_weights.push(
					weight
				);
			} 
		}
			
		let gender = Gender::new();
		let identity = Uuid::new_v4();
		let mut ratings: Vec<f32> = Vec::new();
			
		// generate random ratings based on the given complexity
		for _ in 0..preference_complexity {
			ratings.push(
				rng.gen_range(1.0..=10.0)
			);
		}
		
		return Individual {
			identity: identity.to_string(), 
			gender: gender, 
			preference_weights: predefined_weights,
			ratings: ratings,
			blacklist: Vec::new(),
			candidate: None,
			candidate_score: None
		};
	}
	
	/// calculate the score of this individual to the other
	pub fn score(
		&self, 
		matcher: &Individual
	) -> Result<f32, Box<dyn std::error::Error>> {
		
		if self.preference_weights.len() != matcher.ratings.len() {
			return Err(
				"Twos' predefined weights and ratings do not match.".into()
			);
		}
		
		let score = self.preference_weights
			.iter()
			.zip(
				matcher.ratings.iter()
			)
			.map(|(w, r)| w * r)
			.sum();
		
		return Ok(score); 
	}
	
}

#[derive(Debug)]
pub struct Sample {
	pub male_population: Vec<Individual>,
	pub female_population: Vec<Individual> 
}

impl std::fmt::Display for Sample {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "Male Population: ")?;
		for male_individual in &self.male_population {
			writeln!(f, "===================")?;
			writeln!(f, "{}", male_individual)?;
		}
		
		writeln!(f, "Female Population: ")?;
		for female_individual in &self.female_population {
			writeln!(f, "===================")?;
			writeln!(f, "{}", female_individual)?;
		}
		
		return Ok(());
	}
}

impl Sample {
	/// initiate a population for simulating match-making
	pub fn new(
		population_size: i64,
		preference_complexity: i8,
		specified_predefined_weights: Option<Vec<f32>>
	) -> Self {
		let mut male_population: Vec<Individual> = Vec::new();
		let mut female_population: Vec<Individual> = Vec::new();
		
		let progress_bar = indicatif::ProgressBar::new(
			population_size as u64
		);
		progress_bar.set_style(indicatif::ProgressStyle::with_template(
			"{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})"
		)
	        .unwrap()
	        .with_key(
				"eta", 
				|state: &indicatif::ProgressState, w: &mut dyn std::fmt::Write| write!(
					w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
				)
	        .progress_chars("#>-"));
		
		let mut progress_bar_position = 0;
		
		println!("Preparing the simulation data...");
		
		for _ in 0..population_size {
			let individual = Individual::new(
				preference_complexity, 
				specified_predefined_weights.clone()
			);
			
			if individual.gender == Gender::Female {
				female_population.push(
					individual
				);
			} else {
				male_population.push(
					individual
				);
			}
			
			progress_bar_position += 1;
			progress_bar.set_position(progress_bar_position);
		}
		
		progress_bar.finish_with_message(
			format!(
				"Simulation data preparation has completed in {}", 
				progress_bar.elapsed().as_secs()
			)
		);
		
		return Sample {
			male_population: male_population, 
			female_population: female_population
		};
	}
	
	/// process the action after the two gets matched
	pub fn liked(
		female_individual: &mut Individual, 
		male_individual: &mut Individual, 
		score: f32
	) {
		female_individual.candidate = Some(
			male_individual.identity.clone()
		);
		female_individual.candidate_score = Some(
			score
		);
		
		male_individual.candidate = Some(
			female_individual.identity.clone()
		);
		male_individual.candidate_score = Some(
			score
		);
	}
	
	pub fn match_making(&mut self) -> Result<(), Box<dyn std::error::Error>> {
		
		let progress_bar_male = indicatif::ProgressBar::new(
			self.male_population.len() as u64
		);
		progress_bar_male.set_style(indicatif::ProgressStyle::with_template(
			"{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})"
		)
	        .unwrap()
	        .with_key(
				"eta", 
				|state: &indicatif::ProgressState, w: &mut dyn std::fmt::Write| write!(
					w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
				)
	        .progress_chars("#>-"));
		
		let mut progress_bar_male_position = 0;
		
		println!("Simulating...");
		
		for male_individual in &mut self.male_population {
			for female_individual in &mut self.female_population {
				
				if male_individual.blacklist.contains(&&female_individual.identity) {
					continue;
				}
				
				let score = female_individual
					.score(male_individual)?;
				
				// if the score is smaller than the previous candidate, 
				// the male is going to put the female to a blacklist,
				// and the female will do the same
				if female_individual.candidate_score.is_some() {
					
					if score < female_individual.candidate_score.unwrap() {
						male_individual.blacklist.push(
							female_individual.identity.clone()
						);
					} else {
						Sample::liked(female_individual, male_individual, score);
						break;
					}
					
				} else {
					Sample::liked(female_individual, male_individual, score);
					break;
				}
			}
			
			progress_bar_male_position += 1;
			progress_bar_male.set_position(progress_bar_male_position);
		}
		
		progress_bar_male.finish_with_message(
			format!(
				"Simulation completed in {} secs", 
				progress_bar_male.elapsed().as_secs()
			)
		);
		
		return Ok(());
	}
	
	// display matched pairs
	pub fn display_matches(&self) {
		
		// store the female individuals that have no matches 
		let mut no_match_female_individuals: Vec<&Individual> = Vec::new(); 
		
		for male_individual in &self.male_population {
			
			// a vec that is used to store the reference of matched individuals
			let mut matches: Vec<&Individual> = Vec::new();
			
			for female_individual in &self.female_population {
				if Some(male_individual.identity.clone()) == female_individual.candidate {
					matches.push(female_individual);
				} else if female_individual.candidate.is_none() {
					no_match_female_individuals.push(female_individual);
				}
			}
			
			// print the male individual's information 
			println!("============================================");
			println!("Matches of Male {}", male_individual.identity);
			println!("Below is Male {}'s information", male_individual.identity);
			println!("{}", male_individual);
			println!("");
			
			// print the male matches in the `male_individual`
			if matches.is_empty() {
				println!("No match!");
			}
			
			for matched in matches {
				println!("{}", matched);
			}
		}
		
	}
	
	pub fn display_statistics(&self) {
		
		// store the match information
	    let mut no_match_female_individuals: Vec<&Individual> = Vec::new(); 
	    let mut no_match_male_individuals: Vec<&Individual> = Vec::new();
	    
	    let mut matched_male_individuals: Vec<&Individual> = Vec::new();
	    let mut matched_female_individuals: Vec<&Individual> = Vec::new();
	    
	    for male_individual in &self.male_population {
	        let mut matched = false;
	        
	        for female_individual in &self.female_population {
	            if let Some(candidate) = &female_individual.candidate {
	                if male_individual.identity == *candidate {
	                    matched_female_individuals.push(female_individual);
	                    matched_male_individuals.push(male_individual);
	                    matched = true;
	                    break; // Stop further checking if matched
	                }
	            }
	        }
	        
	        if !matched {
	            no_match_male_individuals.push(male_individual);
	        }
	    }
	    
	    for female_individual in &self.female_population {
	        if female_individual.candidate.is_none() || 
	           !matched_female_individuals.contains(&female_individual) {
	            no_match_female_individuals.push(female_individual);
	        }
	    }
	    
	    println!("Statistics:");
	    println!("Males that do not have a match: {}/{}", no_match_male_individuals.len(), self.male_population.len());
	    println!("Females that do not have a match: {}/{}", no_match_female_individuals.len(), self.female_population.len());
	    println!("Males that have a match: {}/{}", matched_male_individuals.len(), self.male_population.len());
	    println!("Females that have a match: {}/{}", matched_female_individuals.len(), self.female_population.len());
	    
	    let male_population_size = self.male_population.len();
	    let female_population_size = self.female_population.len();
	    let total_population_size = male_population_size + female_population_size;
	    let total_unmatched_individuals = no_match_male_individuals.len() + no_match_female_individuals.len();
	    
	    println!("Descriptions:");
	    if male_population_size > female_population_size {
	        println!("In this simulation, male population EXCEEDED that of female by {}", male_population_size - female_population_size);
	    } else {
	        println!("In this simulation, male population FEWER that of female by {}", female_population_size - male_population_size);
	    }
	    
	    let unmatched_percentage = (total_unmatched_individuals as f64 / total_population_size as f64) * 100.0;
	    println!("{:.2}% of individuals were never matched.", unmatched_percentage);
		
	}
}

fn main() {
	
	let rounds: i8 = 100;
	
	let mut sample = Sample::new(
		10000, 
		3, 
		// Some(vec![0.7, 0.2, 0.1]),
		None
	);
	
	let mut current_round: i8 = 0;
	for _ in 0..rounds {
		let start = std::time::Instant::now();
		
		sample.match_making().unwrap();
		sample.display_statistics();
		
		current_round += 1;
		
		println!(
			"Simulation completed in {} seconds. {}/{}", 
			start.elapsed().as_secs(),
			current_round,
			rounds,
		);
		// sample.display_matches();
	}
	
}