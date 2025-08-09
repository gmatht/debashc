use crate::ast::*;

pub struct FrenchGenerator;

impl FrenchGenerator {
    pub fn new() -> Self { Self }

    pub fn generate(&mut self, commands: &[Command]) -> String {
        let mut out = String::new();
        for c in commands {
            out.push_str(&self.decrire_commande(c));
        }
        while out.ends_with("\n\n") { out.pop(); }
        if !out.ends_with('\n') { out.push('\n'); }
        out
    }

    fn decrire_commande(&self, c: &Command) -> String {
        match c {
            Command::Simple(cmd) => {
                if cmd.name == "echo" {
                    if cmd.args.is_empty() { "Afficher une ligne vide.\n".to_string() } else { format!("Afficher: {}.\n", cmd.args.join(" ")) }
                } else {
                    if cmd.args.is_empty() { format!("Exécuter '{}'.\n", cmd.name) } else { format!("Exécuter '{}' avec les arguments '{}'.\n", cmd.name, cmd.args.join(" ")) }
                }
            }
            Command::Pipeline(p) => {
                let mut s = String::from("Créer un pipeline: ");
                let parts: Vec<String> = p.commands.iter().map(|pc| match pc { Command::Simple(sc) => sc.name.clone(), _ => String::from("commande") }).collect();
                s.push_str(&parts.join(" | "));
                s.push_str(".\n");
                s
            }
            Command::If(ifc) => {
                let mut s = String::from("Si la condition est vraie, alors: \n");
                s.push_str(&self.decrire_commande(&ifc.then_branch));
                if let Some(e) = &ifc.else_branch {
                    s.push_str("Sinon: \n");
                    s.push_str(&self.decrire_commande(e));
                }
                s
            }
            Command::While(_) => String::from("Répéter tant que la condition est vraie.\n"),
            Command::For(_) => String::from("Boucler sur des éléments.\n"),
            Command::Function(f) => format!("Définir la fonction '{}'.\n", f.name),
            Command::Subshell(_) => String::from("Exécuter dans un sous-shell.\n"),
            Command::Background(cmd) => {
                let mut s = String::from("Exécuter en arrière-plan: \n");
                s.push_str(&self.decrire_commande(cmd));
                s
            }
            Command::Block(block) => {
                let mut s = String::from("Exécuter un bloc de commandes:\n");
                for c in &block.commands { s.push_str(&self.decrire_commande(c)); }
                s
            }
            Command::BlankLine => String::from("\n"),
        }
    }
}



