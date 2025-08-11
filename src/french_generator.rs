use crate::ast::*;

pub struct FrenchGenerator;

impl FrenchGenerator {
    pub fn new() -> Self { Self }

    pub fn generate(&mut self, commands: &[Command]) -> String {
        let mut out = String::new();
        for c in commands {
            out.push_str(&self.generate_command(c));
        }
        while out.ends_with('\n') { out.pop(); }
        out
    }

    fn generate_command(&mut self, c: &Command) -> String {
        match c {
            Command::Simple(cmd) => self.generate_simple_command(cmd),
            Command::ShoptCommand(cmd) => self.generate_shopt_command(cmd),
            Command::TestExpression(test_expr) => self.generate_test_expression(test_expr),
            Command::Pipeline(pipeline) => self.generate_pipeline(pipeline),
            Command::If(if_stmt) => self.generate_if_statement(if_stmt),
            Command::While(while_loop) => self.generate_while_loop(while_loop),
            Command::For(for_loop) => self.generate_for_loop(for_loop),
            Command::Function(func) => self.generate_function(func),
            Command::Subshell(cmd) => self.generate_subshell(cmd),
            Command::Background(cmd) => self.generate_background(cmd),
            Command::Block(block) => self.generate_block(block),
            Command::BlankLine => String::from("\n"),
        }
    }

    fn generate_simple_command(&mut self, cmd: &SimpleCommand) -> String {
        self.decrire_commande(&Command::Simple(cmd.clone()))
    }

    fn generate_shopt_command(&mut self, cmd: &ShoptCommand) -> String {
        self.decrire_commande(&Command::ShoptCommand(cmd.clone()))
    }

    fn generate_test_expression(&mut self, test_expr: &TestExpression) -> String {
        let mut output = String::new();
        
        // Handle test modifiers if they're set
        if test_expr.modifiers.extglob {
            output.push_str("Activer le globbing étendu.\n");
        }
        if test_expr.modifiers.nocasematch {
            output.push_str("Activer la correspondance insensible à la casse.\n");
        }
        if test_expr.modifiers.globstar {
            output.push_str("Activer la correspondance de motifs globstar.\n");
        }
        if test_expr.modifiers.nullglob {
            output.push_str("Activer la correspondance de motifs nullglob.\n");
        }
        if test_expr.modifiers.failglob {
            output.push_str("Activer la correspondance de motifs failglob.\n");
        }
        if test_expr.modifiers.dotglob {
            output.push_str("Activer la correspondance de motifs dotglob.\n");
        }
        
        // Generate the test expression description
        output.push_str(&format!("Évaluer l'expression de test : {}.\n", test_expr.expression));
        
        output
    }

    fn generate_pipeline(&mut self, pipeline: &Pipeline) -> String {
        self.decrire_commande(&Command::Pipeline(pipeline.clone()))
    }

    fn generate_if_statement(&mut self, if_stmt: &IfStatement) -> String {
        self.decrire_commande(&Command::If(if_stmt.clone()))
    }

    fn generate_while_loop(&mut self, while_loop: &WhileLoop) -> String {
        self.decrire_commande(&Command::While(while_loop.clone()))
    }

    fn generate_for_loop(&mut self, for_loop: &ForLoop) -> String {
        self.decrire_commande(&Command::For(for_loop.clone()))
    }

    fn generate_function(&mut self, func: &Function) -> String {
        self.decrire_commande(&Command::Function(func.clone()))
    }

    fn generate_subshell(&mut self, cmd: &Command) -> String {
        self.decrire_commande(&Command::Subshell(Box::new(cmd.clone())))
    }

    fn generate_background(&mut self, cmd: &Command) -> String {
        self.decrire_commande(&Command::Background(Box::new(cmd.clone())))
    }

    fn generate_block(&mut self, block: &Block) -> String {
        self.decrire_commande(&Command::Block(block.clone()))
    }

    fn decrire_commande(&self, c: &Command) -> String {
        match c {
            Command::Simple(cmd) => {
                if cmd.name == "echo" {
                    if cmd.args.is_empty() { "Afficher une ligne vide.\n".to_string() } else { format!("Afficher: {}.\n", cmd.args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>().join(" ")) }
                } else {
                    if cmd.args.is_empty() { format!("Exécuter '{}'.\n", cmd.name) } else { format!("Exécuter '{}' avec les arguments '{}'.\n", cmd.name, cmd.args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>().join(" ")) }
                }
            }
            Command::ShoptCommand(cmd) => {
                if cmd.enable {
                    format!("Activer l'option shell '{}'.\n", cmd.option)
                } else {
                    format!("Désactiver l'option shell '{}'.\n", cmd.option)
                }
            }
            Command::Pipeline(p) => {
                let mut s = String::from("Créer un pipeline: ");
                let parts: Vec<String> = p.commands.iter().map(|pc| match pc { Command::Simple(sc) => sc.name.to_string(), _ => String::from("commande") }).collect();
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
            Command::TestExpression(_) => String::from("Évaluer une expression de test.\n"),
        }
    }
}



