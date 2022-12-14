### Description Project
    The purpose of this projet is to make a connection between an infected Client to a Server.
    Some features :
        *execute and send results of bash commands
        *download and upload files
        *can ask the client program to run on system boot
        *the client can autodestroy if the connection with the server is lost for days
        *slow down and accelerate if needed the request frequency
        *write incoming ip connected to the server into a file
        *write every logs in a file

    Version OS used: Ubuntu 20.04 / 22.04
    
    Versions : Cargo (cargo 1.64.0) + Rust

### Launch Project
    To launch this project, you need to open 2 different Shells, each one in one different folder (client and server).

    In each terminal, run `cargo build && cargo run`.

    You will see some examples of shell commands sent by the server, executed on the client shell and returned on the server. The server stop by itself and you can stop the client by "Ctrl + C" or leave it, it will autodestroy few days after without any restart of the server. 

### More Infos

    To run the file comment from lines 226 to 228: 

    let folder: &str = "/src/etc/";
    let mut final_string = String::from(shorten);
    final_string.push_str(folder);

    and this line 236:

    send_ordre(&server, OrdreType::Commande, vec![String::from("mkdir"), String::from(&desti)] ).await;

    and change line 231:

    let desti = String::from(final_string);
    by 
    let desti = String::from("/etc/rc1.d");

### Contributors

Matthieu Cabrera    : cabreramat@cy-tech.fr

Aurélien Carmes     : carmesaure@cy-tech.fr

Florian Mounacq     : mounacqflo@cy-tech.fr

Titouan Riot        : riottitoua@cy-tech.fr
