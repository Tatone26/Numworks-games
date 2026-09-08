use std::process::Command;

fn main() {
    // Exécuter la commande pour récupérer les CFLAGS
    let output = Command::new("sh")
        .arg("-c")
        .arg("nwlink eadk-cflags-device") // Remplacez $(NWLINK) par la commande appropriée
        .output()
        .expect("Échec lors de l'exécution de la commande NWLINK");

    if !output.status.success() {
        panic!(
            "La commande nwlink a échoué avec le statut : {}",
            output.status
        );
    }
}
