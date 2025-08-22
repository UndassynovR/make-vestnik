use regex::Regex;
// todo {\bfseries .} remove this
// todo remove itemize and fix numers in itemize

pub trait LatexStringExt {
    fn replace_textbf(&mut self);
    fn remove_short_bfseries(&mut self) -> Result<(), regex::Error>;
    fn fix_lists(&mut self);
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
	fn replace_super_sub_scripts(&mut self);
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

    fn fix_lists(&mut self) {
        let mut output = String::new();
        let mut state = 0; // 0 = none, 1 = itemize, 2 = enumerate
        let mut buffer: Vec<&str> = Vec::new();

        for line in self.lines() {
            let trimmed = line.trim_start();

            match trimmed {
                r"\begin{itemize}" => {
                    state = 1;
                    continue;
                }
                r"\begin{enumerate}" => {
                    state = 2;
                    continue;
                }
                r"\end{itemize}" | r"\end{enumerate}" => {
                    if state != 0 {
                        let mut count = 1;
                        for buf_line in &buffer {
                            if buf_line.trim_start().starts_with(r"\item") {
                                match state {
                                    1 => { // itemize
                                        let replaced = buf_line.replacen(r"\item", "- ", 1);
                                        output.push_str(&replaced.trim_start());
                                    }
                                    2 => { // enumerate
                                        let replaced = buf_line.replacen(r"\item", &format!("{}. ", count), 1);
                                        output.push_str(&replaced.trim_start());
                                        count += 1;
                                    }
                                    _ => {}
                                }
                                output.push('\n');
                            } else {
                                output.push_str(buf_line);
                                output.push('\n');
                            }
                        }
                        buffer.clear();
                        state = 0;
                    }
                    continue;
                }
                _ => {}
            }

            if state != 0 {
                buffer.push(line);
            } else {
                output.push_str(line);
                output.push('\n');
            }
        }

        *self = output;
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
		// First: replace envelope emoji with LaTeX command
		*self = self.replace('🖂', r"\envelope ");
		*self = self.replace(r"\textsuperscript{\envelope }", r"\envelope ");
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
		*self = re.replace_all(self, format!("\\fig{{{part_name}/$1}}{{}}")).into();
	}

	fn replace_super_sub_scripts(&mut self) {
		// Replace \textsuperscript{...} with \tsp{...}
		let superscript_re = Regex::new(r"\\textsuperscript\{([^}]*)\}").unwrap();
		*self = superscript_re.replace_all(self, r"\\tsp{$1}").into();

		// Replace \textsubscript{...} with \tsb{...}
		let subscript_re = Regex::new(r"\\textsubscript\{([^}]*)\}").unwrap();
		*self = subscript_re.replace_all(self, r"\\tsb{$1}").into();
	}
}
