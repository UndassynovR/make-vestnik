use regex::Regex;
// todo {\bfseries .} remove this
// todo remove itemize and fix numers in itemize

pub trait LatexStringExt {
    fn replace_textbf(&mut self);
    fn remove_short_bfseries(&mut self) -> Result<(), regex::Error>;
    fn fix_enumerate(&mut self) -> Result<(), regex::Error>;
    fn fix_itemize(&mut self) -> Result<(), regex::Error>;
    fn fix_number_spacing(&mut self) -> Result<(), regex::Error>;
    fn remove_tag(&mut self, tag: &str);
    fn comment_out_tables(&mut self);
    fn change_latex_quotes(&mut self);
    fn replace_envelopes(&mut self);
    fn remove_tightlists(&mut self);
    fn unindent(&mut self);
    fn split_articles(&self) -> Vec<String>;
    fn replace_bullets(&mut self);
    fn fix_images(&mut self, part_name: &str);
}

impl LatexStringExt for String {
    fn replace_textbf(&mut self) {
        *self = self.replace("\\textbf{", "{\\bfseries ")
    }

    fn remove_short_bfseries(&mut self) -> Result<(), regex::Error> {
        let re = Regex::new(r"\{\\bfseries ([^\s])}")?;
        *self = re.replace_all(self, "$1").to_string();
        Ok(())
    }

    fn fix_enumerate(&mut self) -> Result<(), regex::Error> {
        let re_item = Regex::new(r"\\item\s+")?;
        let re_numbers = Regex::new(r"\n((?:\d+\.)+)\s*")?;
        let re_dot = Regex::new(r"\.( )(\d)")?;

        let mut output = String::new();
        let mut inside_enum = false;
        let mut buffer = Vec::new();

        for line in self.lines() {
            let trimmed = line.trim_start();

            if trimmed == r"\begin{enumerate}" {
                inside_enum = true;
                continue;
            }

            if trimmed == r"\def\labelenumi{\arabic{enumi}.}" {
                continue;
            }

            if trimmed == r"\end{enumerate}" {
                inside_enum = false;
                // process buffered block
                let mut block = buffer.join("\n");
                block = re_item.replace_all(&block, "\n1. ").to_string();
                block = re_numbers.replace_all(&block, "\n$1 ").to_string();
                block = re_dot.replace_all(&block, ".$2").to_string();
                output.push_str(&block);
                buffer.clear();
                continue;
            }

            if inside_enum {
                buffer.push(line);
            } else {
                output.push_str(line);
                output.push('\n');
            }
        }

        *self = output;
        Ok(())
    }

    fn fix_itemize(&mut self) -> Result<(), regex::Error> {
        let re_item = Regex::new(r"\\item\s+")?;

        let mut output = String::new();
        let mut inside_itemize = false;
        let mut buffer = Vec::new();

        for line in self.lines() {
            let trimmed = line.trim_start();

            if trimmed == r"\begin{itemize}" {
                inside_itemize = true;
                continue;
            }

            if trimmed == r"\end{itemize}" {
                inside_itemize = false;
                // process the collected block
                let mut block = buffer.join("\n");
                block = re_item.replace_all(&block, "\n- ").to_string();
                output.push_str(&block);
                buffer.clear();
                continue;
            }

            if inside_itemize {
                buffer.push(line);
            } else {
                output.push_str(line);
                output.push('\n');
            }
        }

        *self = output;
        Ok(())
    }

    fn fix_number_spacing(&mut self) -> Result<(), regex::Error> {
        let re_numbers = Regex::new(r"\n((?:\d+\.)+)\s*")?;
        let re_dot = Regex::new(r"\.( )(\d)")?;

        let mut result = self.clone();
        result = re_numbers.replace_all(&result, "\n$1 ").to_string();
        result = re_dot.replace_all(&result, ".$2").to_string();

        *self = result;
        Ok(())
    }

    fn remove_tag(&mut self, tag: &str) {
        let mut output = String::new();
        let chars: Vec<char> = self.chars().collect();
        let tag_pattern = format!("\\{}{{", tag);
        let tag_chars: Vec<char> = tag_pattern.chars().collect();

        let mut i = 0;
        while i < chars.len() {
            if chars[i..].starts_with(&tag_chars) {
                i += tag_chars.len();
                let mut brace_level = 1;
                while i < chars.len() {
                    match chars[i] {
                        '{' => brace_level += 1,
                        '}' => {
                            brace_level -= 1;
                            i += 1;
                            if brace_level == 0 {
                                break;
                            }
                            continue;
                        }
                        _ => {}
                    }
                    i += 1;
                }
            } else {
                output.push(chars[i]);
                i += 1;
            }
        }

        *self = output;
    }

    fn comment_out_tables(&mut self) {
        let mut inside_table = false;
        let mut result = Vec::new();

        for line in self.lines() {
            if line.starts_with(r"\begin{longtable}[]{@{}") {
                inside_table = true;
                result.push(format!("%% {}", line));
            } else if line.starts_with(r"\end{longtable}") && inside_table {
                inside_table = false;
                result.push(format!("%% {}", line));
            } else if inside_table {
                result.push(format!("%% {}", line));
            } else {
                result.push(line.to_string());
            }
        }

        *self = result.join("\n");
    }

    fn change_latex_quotes(&mut self) {
        let mut replaced = self.replace(r"\textquotesingle", "'");
        replaced = replaced.replace(r"\textquotedbl", "\"");
        *self = replaced;
    }

    fn replace_envelopes(&mut self) {
        *self = self.replace('🖂', r"\envelope ");
    }

    fn remove_tightlists(&mut self) {
        *self = self.replace(r"\tightlist", "");
    }

    fn unindent(&mut self) {
        *self = self
            .lines()
            .map(|line| line.trim_start()) // Remove all leading whitespace
            .collect::<Vec<_>>()
            .join("\n");
    }

    fn replace_bullets(&mut self) {
        // Regex: start of line (^), optional whitespace (\s*), bullet (•)
        let re = Regex::new(r"(?m)^(\s*)•").unwrap();
        *self = re.replace_all(self, "$1-").into();
    }

    fn split_articles(&self) -> Vec<String> {
        let re_wrap =
            Regex::new(r"\s*(?:\{\\bfseries\s+)?((?:IRSTI|ҒТАМР|МРНТИ|ГРНТИ)[0-9. ]*)\}?").unwrap();
        let modified = re_wrap.replace_all(self, r"\\id{$1}{}").into_owned();

        let re_split = Regex::new(r"\\id\{(?:МРНТИ|IRSTI|ҒТАМР|ГРНТИ)[0-9 .,]*\}\{\}").unwrap();
        let mut articles = Vec::new();
        let mut last_index = 0;

        let markers: Vec<_> = re_split.find_iter(&modified).collect();

        for (i, mat) in markers.iter().enumerate() {
            if i > 0 {
                let article = &modified[last_index..mat.start()];
                articles.push(article.trim().to_string());
            }
            last_index = mat.start();
        }

        if last_index < modified.len() {
            let article = &modified[last_index..];
            articles.push(article.trim().to_string());
        }

        articles
    }

    fn fix_images(&mut self, part_name: &str) {
        let re = Regex::new(
            r"\\includegraphics\[[^]]*\]\{media/([^}/\\]+?)(?:\.(?:png|jpe?g|pdf|webp|wmf|emf))?\}",
        )
        .unwrap();
        *self = re.replace_all(self, format!("\n\\begin{{figure}}[H]\n\t\\centering\n\t\\includegraphics[width=0.8\\textwidth]{{media/{part_name}/$1}}\n\t\\caption*{{}}\n\\end{{figure}}\n")).into();
    }
}
