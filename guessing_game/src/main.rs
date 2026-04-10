use std::cmp::Ordering;
use std::fs;
use std::io;
use rand::Rng;
use colored::*;

fn main() {
    let score_data =fs::read_to_string("score.txt").unwrap();
    let score_data = score_data.split(",");
    let mut player_score: u32 = score_data.clone().nth(0).unwrap_or("0").trim().parse().unwrap_or(0);
    let mut computer_score: u32 = score_data.clone().nth(1).unwrap_or("0").trim().parse().unwrap_or(0);
    println!("--- Guess the number! ---");
    println!("Current Score:");
    println!("Player: {}", player_score);
    println!("Computer: {}", computer_score);

    let secret_number = rand::thread_rng().gen_range(1..=100);

    let mut attempt = 5;

    loop {
        if attempt == 0 {
            println!("{} {}", "You have run out of attempts. The number was".red(), secret_number);
            computer_score += 1;
            break;
        }

        println!("Please enter your guess:");

        let mut guess = String::new();

        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");

        let guess: u32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Please enter a valid number!");
                continue;  
            },
        };

        println!("You guessed: {}", guess);

        match guess.cmp(&secret_number) {
            Ordering::Less => println!("{} {} {}", "Too small!".yellow(), "attempts left.".yellow(), attempt - 1),
            Ordering::Greater => println!("{} {} {}", "Too big!".yellow(), "attempts left.".yellow(), attempt - 1),
            Ordering::Equal => {
                println!("{} {}", "Congratulations! You guessed the number!".green(), secret_number);
                player_score += 1;
                break;
            }
        }

        attempt -= 1;
    }

    fs::write("score.txt", format!("{},{}", player_score, computer_score)).unwrap();
}
