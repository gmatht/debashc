use crate::ast::*;
use crate::generator::Generator;

pub fn generate_curl_command(generator: &mut Generator, cmd: &SimpleCommand) -> String {
    let mut output = String::new();
    
    // curl command syntax: curl [options] [URL...]
    let mut url = "".to_string();
    let mut method = "GET".to_string();
    let mut headers = Vec::new();
    let mut data = "".to_string();
    let mut output_file = "".to_string();
    let mut follow_redirects = false;
    let mut silent_mode = false;
    let mut verbose_mode = false;
    let mut timeout = 0;
    
    // Parse curl options
    let mut i = 0;
    while i < cmd.args.len() {
        if let Word::Literal(arg_str) = &cmd.args[i] {
            match arg_str.as_str() {
                "-X" | "--request" => {
                    if i + 1 < cmd.args.len() {
                        if let Some(next_arg) = cmd.args.get(i + 1) {
                            method = generator.word_to_perl(next_arg);
                            i += 1;
                        }
                    }
                }
                "-H" | "--header" => {
                    if i + 1 < cmd.args.len() {
                        if let Some(next_arg) = cmd.args.get(i + 1) {
                            headers.push(generator.word_to_perl(next_arg));
                            i += 1;
                        }
                    }
                }
                "-d" | "--data" => {
                    if i + 1 < cmd.args.len() {
                        if let Some(next_arg) = cmd.args.get(i + 1) {
                            data = generator.word_to_perl(next_arg);
                            i += 1;
                        }
                    }
                }
                "-o" | "--output" => {
                    if i + 1 < cmd.args.len() {
                        if let Some(next_arg) = cmd.args.get(i + 1) {
                            output_file = generator.word_to_perl(next_arg);
                            i += 1;
                        }
                    }
                }
                "-L" | "--location" => follow_redirects = true,
                "-s" | "--silent" => silent_mode = true,
                "-v" | "--verbose" => verbose_mode = true,
                "--connect-timeout" => {
                    if i + 1 < cmd.args.len() {
                        if let Some(next_arg) = cmd.args.get(i + 1) {
                            if let Word::Literal(timeout_str) = next_arg {
                                if let Ok(timeout_num) = timeout_str.parse::<i32>() {
                                    timeout = timeout_num;
                                }
                            }
                            i += 1;
                        }
                    }
                }
                _ => {
                    if !arg_str.starts_with('-') && url.is_empty() {
                        url = generator.word_to_perl(&cmd.args[i]);
                    }
                }
            }
        } else {
            if url.is_empty() {
                url = generator.word_to_perl(&cmd.args[i]);
            }
        }
        i += 1;
    }
    
    if url.is_empty() {
        output.push_str("die \"curl: missing URL\\n\";\n");
    } else {
        output.push_str("use LWP::UserAgent;\n");
        output.push_str("use HTTP::Request;\n");
        output.push_str("use HTTP::Headers;\n");
        
        // Create UserAgent
        output.push_str("my $ua = LWP::UserAgent->new;\n");
        
        // Set timeout if specified
        if timeout > 0 {
            output.push_str(&format!("$ua->timeout({});\n", timeout));
        }
        
        // Set redirects if specified
        if follow_redirects {
            output.push_str("$ua->max_redirect(5);\n");
        }
        
        // Create headers
        if !headers.is_empty() {
            output.push_str("my $headers = HTTP::Headers->new;\n");
            for header in &headers {
                output.push_str(&format!("$headers->header({});\n", header));
            }
        }
        
        // Create request
        output.push_str(&format!("my $request = HTTP::Request->new('{}', {});\n", method, url));
        
        if !headers.is_empty() {
            output.push_str("$request->headers($headers);\n");
        }
        
        if !data.is_empty() {
            output.push_str(&format!("$request->content({});\n", data));
        }
        
        // Make request
        output.push_str("my $response = $ua->request($request);\n");
        
        // Handle response
        output.push_str("if ($response->is_success) {\n");
        if !output_file.is_empty() {
            output.push_str(&format!("if (open(my $fh, '>', {})) {{\n", output_file));
            output.push_str("print $fh $response->content;\n");
            output.push_str("close($fh);\n");
            output.push_str(&format!("print \"Content saved to {}\\n\";\n", output_file));
            output.push_str("} else {\n");
            output.push_str(&format!("die \"curl: Cannot write to {}: $!\\n\";\n", output_file));
            output.push_str("}\n");
        } else {
            if !silent_mode {
                output.push_str("print $response->content;\n");
            }
        }
        output.push_str("} else {\n");
        if verbose_mode {
            output.push_str(&format!("print STDERR \"HTTP {}: {}\\n\";\n", 
                "$response->code", "$response->message"));
        }
        output.push_str(&format!("die \"curl: HTTP error: {} {}\\n\";\n", 
            "$response->code", "$response->message"));
        output.push_str("}\n");
    }
    
    output
}
