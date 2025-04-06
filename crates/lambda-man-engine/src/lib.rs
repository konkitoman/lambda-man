// [Lambda Calculus Calculator](https://lambdacalc.io/) this was used for testing.

// x:x is buffer
// x:y:x is True
// x:y:y is False
// f:x:x is 0
// f:x:f x is 1
// f:x:f (f x) is 2
// f:x:f (f (f x) is 3
// f:x:f (f (f (f x) is 4
// f:x:f (f (f (f (f (x) is 5
// f:x:f (f (f (f (f (f (x) is 6
// f:x:f (f (f (f (f (f (f x) is 7
// f:x:f (f (f (f (f (f (f (f x) is 8
// f:x:f (f (f (f (f (f (f (f (f x) is 9
// n:f:x:f (n f x) is succ

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Group(Vec<Expr>),
    Def(Box<Expr>),
    Relative(u32),
    Label(String),
}

impl Expr {
    pub fn format(&self, depth: u32) -> String {
        let mut out = String::default();

        match self {
            Expr::Group(exprs) => {
                out.push('(');
                for (i, expr) in exprs.iter().enumerate() {
                    out.push_str(&expr.format(depth));
                    if i != exprs.len() - 1 {
                        out.push(' ');
                    }
                }
                out.push(')');
            }
            Expr::Def(expr) => {
                out.push_str(&format!("{}:", var_name(depth)));
                out.push_str(&expr.format(depth + 1));
            }
            Expr::Relative(id) => {
                if let Some(id) = depth.checked_sub(*id + 1) {
                    out.push_str(&var_name(id));
                } else {
                    out.push_str(&format!("!{id}!"));
                }
            }
            Expr::Label(name) => {
                out.push('\'');
                out.push_str(name);
            }
        }

        out
    }

    pub fn format_highlightd(&self, depth: u32, at: &[usize], ansi_color: &str) -> String {
        self._format_highlight(depth, vec![], at, ansi_color)
    }

    fn _format_highlight(&self, depth: u32, path: Vec<usize>, at: &[usize], ansi: &str) -> String {
        let mut out = String::default();

        let highlighted = path == at;

        if highlighted {
            out.push('\x1B');
            out.push('[');
            out.push_str(ansi);
            out.push('m');
        }

        match self {
            Expr::Group(exprs) => {
                out.push('(');
                for (i, expr) in exprs.iter().enumerate() {
                    let mut path = path.clone();
                    path.push(i);
                    out.push_str(&expr._format_highlight(depth, path, at, ansi));
                    if i != exprs.len() - 1 {
                        out.push(' ');
                    }
                }
                out.push(')');
            }
            Expr::Def(expr) => {
                out.push_str(&format!("{}:", var_name(depth)));
                out.push_str(&expr._format_highlight(depth + 1, path.clone(), at, ansi));
            }
            _ => {
                out.push_str(&self.format(depth));
            }
        }

        if highlighted {
            out.push_str("\x1B[39m");
            out.push_str("\x1B[49m");
        }

        out
    }

    pub fn parse(text: &str) -> Option<Self> {
        let mut error = false;
        let res = Self::_parse(text, &mut 0, &mut error, vec![])?;
        (!error).then_some(res)
    }

    pub fn _parse(
        text: &str,
        i: &mut usize,
        error: &mut bool,
        scope: Vec<(String, u32)>,
    ) -> Option<Self> {
        let mut iterator = text[*i..].chars();
        let ch = iterator.next()?;
        *i += 1;

        if ch.is_whitespace() {
            return Self::_parse(text, i, error, scope);
        }

        if ch == '\'' {
            let mut name = String::default();
            for ch in text[*i..].chars() {
                if ch.is_whitespace() {
                    *i += 1;
                    break;
                }

                match ch {
                    ')' | '(' => break,
                    _ => {
                        *i += 1;
                        name.push(ch);
                    }
                }
            }

            return Some(Expr::Label(name));
        }

        if ch.is_alphabetic() {
            let mut name = String::default();
            name.push(ch);

            for ch in iterator {
                if ch.is_whitespace() {
                    *i += 1;
                    break;
                }

                match ch {
                    ':' => {
                        let mut scope = scope.clone();
                        for v in scope.iter_mut() {
                            v.1 += 1;
                        }
                        scope.push((name.clone(), 0));
                        *i += 1;
                        let e = Expr::_parse(text, i, error, scope)?;
                        return Some(Expr::Def(Box::new(e)));
                    }
                    ')' | '(' => break,
                    _ => {
                        if ch.is_alphabetic() {
                            name.push(ch);
                        }
                        *i += 1;
                    }
                }
            }

            for ii in 0..scope.len() {
                let (var_name, id) = &scope[scope.len() - (ii + 1)];
                if var_name.as_str() == name {
                    return Some(Expr::Relative(*id));
                }
            }

            *error = true;

            eprintln!("{text}");
            eprintln!("{:1$}^", " ", *i - 1);
            eprintln!("Cannot find name `{name}` in {scope:?}");

            return None;
        }

        if ch == ')' {
            return None;
        }

        if ch == '(' {
            let mut exprs = Vec::default();
            while let Some(expr) = Self::_parse(text, i, error, scope.clone()) {
                exprs.push(expr);
            }
            return Some(Expr::Group(exprs));
        }

        None
    }

    pub fn replace_relative(&mut self, depth: u32, value: Expr) {
        match self {
            Expr::Group(exprs) => {
                for expr in exprs {
                    expr.replace_relative(depth, value.clone());
                }
            }
            Expr::Def(expr) => expr.replace_relative(depth + 1, value.clone()),
            Expr::Relative(id) => {
                let offset = (*id as i64) - (depth as i64);
                if offset == 0 {
                    *self = value;
                    self.update(0, depth);
                } else if offset >= 0 {
                    *id -= 1;
                }
            }
            _ => {}
        }
    }

    pub fn update(&mut self, depth: u32, at: u32) {
        match self {
            Expr::Group(exprs) => {
                for expr in exprs {
                    expr.update(depth, at);
                }
            }
            Expr::Def(expr) => expr.update(depth + 1, at),
            Expr::Relative(id) => {
                let offset = (*id as i64) - (depth as i64);
                if offset >= 0 {
                    *id += at;
                }
            }
            Expr::Label(_) => {}
        }
    }

    pub fn simplify(&mut self) {
        let Expr::Group(exprs) = self else {
            if let Expr::Def(expr) = self {
                expr.simplify();
            }
            return;
        };

        if exprs.len() != 1 {
            for expr in exprs {
                expr.simplify();
            }

            return;
        }

        if let Expr::Group(_) = &exprs[0] {
            *self = exprs.remove(0);
            self.simplify();
        } else {
            exprs[0].simplify();
        }
    }

    pub fn is_contained(&self, at: u32) -> bool {
        match self {
            Expr::Group(exprs) => {
                for expr in exprs {
                    if !expr.is_contained(at) {
                        return false;
                    }
                }
                true
            }
            Expr::Def(expr) => expr.is_contained(at + 1),
            Expr::Relative(id) => *id < at,
            Expr::Label(_) => true,
        }
    }

    pub fn find_beta_reductions(&self) -> Vec<(u32, Vec<usize>)> {
        self._find_beta_reductions(vec![], 0, vec![])
    }

    pub fn _find_beta_reductions(
        &self,
        pos: Vec<usize>,
        depth: u32,
        mut has_at: Vec<(u32, Vec<usize>)>,
    ) -> Vec<(u32, Vec<usize>)> {
        let mut out = Vec::default();

        match self {
            Expr::Group(exprs) => {
                {
                    let mut i = exprs.iter();
                    i.next();
                    if i.next().is_some() {
                        let mut at = pos.clone();
                        at.push(1);
                        has_at.push((depth, at));
                    }
                }

                for (i, expr) in exprs.iter().enumerate() {
                    let mut at = pos.clone();
                    at.push(i);

                    for (s, new) in expr._find_beta_reductions(at, depth, has_at.clone()) {
                        out.push((s + 1, new));
                    }
                }
            }

            Expr::Def(expr) => {
                if pos.last().copied() == Some(0) {
                    let mut min_i = 0;
                    let mut min = u32::MAX;

                    'f: for (i, (h_depth, at)) in has_at.iter().enumerate() {
                        let mut posi = pos.iter();
                        let mut diverged = false;
                        for (i, ati) in at.iter().enumerate() {
                            let Some(pi) = posi.next() else {
                                continue 'f;
                            };

                            if pi == ati {
                                continue;
                            } else if *ati == pi + 1 {
                                if i == at.len() - 1
                                    && (posi.next().is_none() || posi.next().is_none())
                                {
                                    diverged = true;
                                }
                                break;
                            }
                        }

                        if !diverged {
                            continue;
                        }

                        if *h_depth <= depth && *h_depth <= min {
                            min = *h_depth;
                            min_i = i
                        }
                    }

                    if min != u32::MAX {
                        has_at.remove(min_i);

                        out.push((0, pos.clone()));
                    }
                }

                for (s, new) in expr._find_beta_reductions(pos, depth + 1, has_at) {
                    out.push((s, new));
                }
            }
            _ => {}
        }

        out
    }

    pub fn beta_reduction_at(&mut self, at: &[usize]) -> bool {
        let mut tree = vec![self];
        let mut depth = 0;

        let mut ati = at.iter();
        {
            let mut index = ati.next().copied();
            loop {
                let Some(i) = index else {
                    break;
                };
                match unsafe {
                    std::mem::transmute::<&mut Expr, &mut Expr>(*tree.last_mut().unwrap())
                } {
                    Expr::Group(new_exprs) => {
                        tree.push(&mut new_exprs[i]);
                        index = ati.next().copied();
                    }
                    Expr::Def(expr) => {
                        depth += 1;
                        tree.push(expr.as_mut());
                    }
                    node => panic!("{}", node.format(depth)),
                }
            }
        }

        let last =
            unsafe { std::mem::transmute::<&mut Expr, &mut Expr>(*tree.last_mut().unwrap()) };

        if let Expr::Def(_) = &last {
        } else {
            eprintln!("Is not DEF");
            return false;
        }

        let mut value = None;
        let mut upper_tree = 2;
        let mut upper_i = 1;

        let len = tree.len();

        while let Some(parent) = tree.get_mut(len - upper_tree) {
            let index = at.get(at.len() - upper_i).copied().unwrap();

            match parent {
                Expr::Def(_) => {
                    panic!(
                        "Using other functions argument., how is this possibile?, this should not be able to happend, how i got here?"
                    );
                }

                Expr::Group(group) => {
                    if group.get(index + 1).is_some() && group[index + 1].is_contained(depth) {
                        value = Some(group.remove(index + 1));
                        break;
                    }
                    upper_i += 1;
                }

                _ => {}
            }

            upper_tree += 1;
            if upper_tree > len {
                return false;
            }
        }

        let Some(value) = value else {
            eprintln!("Cannot find value");
            return false;
        };

        let Expr::Def(mut def) = last.clone() else {
            return false;
        };

        def.replace_relative(0, value);

        *last = *def;

        last.simplify();

        true
    }
}

impl From<usize> for Expr {
    fn from(num: usize) -> Self {
        let mut exprs = Vec::default();

        let mut e = &mut exprs;

        for _ in 0..num {
            let exprs = Vec::default();
            e.push(Expr::Group(exprs));
            let Expr::Group(exprs) = e.last_mut().unwrap() else {
                panic!()
            };
            e = exprs;
            e.push(Expr::Relative(1));
        }

        e.push(Expr::Relative(0));

        Expr::Group(vec![if let Some(Expr::Group(_)) = exprs.first() {
            Expr::Def(Box::new(Expr::Def(Box::new(exprs.remove(0)))))
        } else {
            Expr::Def(Box::new(Expr::Def(Box::new(Expr::Group(exprs)))))
        }])
    }
}

fn var_name(id: u32) -> String {
    match id {
        0 => String::from("a"),
        1 => String::from("b"),
        2 => String::from("c"),
        3 => String::from("d"),
        4 => String::from("e"),
        5 => String::from("f"),
        6 => String::from("g"),
        7 => String::from("h"),
        8 => String::from("i"),
        9 => String::from("j"),
        10 => String::from("k"),
        11 => String::from("l"),
        12 => String::from("m"),
        13 => String::from("n"),
        14 => String::from("o"),
        15 => String::from("p"),
        16 => String::from("q"),
        17 => String::from("r"),
        18 => String::from("s"),
        19 => String::from("t"),
        20 => String::from("u"),
        21 => String::from("v"),
        22 => String::from("w"),
        23 => String::from("x"),
        24 => String::from("y"),
        25 => String::from("z"),
        _ => todo!("Need more names for veriabiles to be displayed."),
    }
}

#[test]
fn boolean() {
    println!("true function");
    let mut expr = Expr::Group(vec![
        Expr::Def(Box::new(Expr::Def(Box::new(Expr::Relative(1))))),
        Expr::Label("TRUE".into()),
        Expr::Label("FALSE".into()),
    ]);

    println!("{}", expr.format(0));
    eprintln!("{:?}", expr.find_beta_reductions());

    expr.beta_reduction_at(&[0]);

    println!("{}", expr.format(0));
    eprintln!("{:?}", expr.find_beta_reductions());

    expr.beta_reduction_at(&[0]);

    println!("{}", expr.format(0));
    eprintln!("{:?}", expr.find_beta_reductions());

    assert_eq!(expr, Expr::parse("('TRUE)").unwrap());

    println!("false function");
    let mut expr = Expr::Group(vec![
        Expr::Def(Box::new(Expr::Def(Box::new(Expr::Relative(0))))),
        Expr::Label("TRUE".into()),
        Expr::Label("FALSE".into()),
    ]);

    println!("{}", expr.format(0));
    eprintln!("{:?}", expr.find_beta_reductions());

    expr.beta_reduction_at(&[0]);

    println!("{}", expr.format(0));
    eprintln!("{:?}", expr.find_beta_reductions());

    expr.beta_reduction_at(&[0]);

    println!("{}", expr.format(0));
    eprintln!("{:?}", expr.find_beta_reductions());

    assert_eq!(expr, Expr::parse("('FALSE)").unwrap());
}

#[test]
fn succ() {
    // n:f:x:f (n f x) is succ
    // n:f:x:(f (n f x)) is succ
    let succ = Expr::Def(Box::new(Expr::Def(Box::new(Expr::Def(Box::new(
        Expr::Group(vec![
            Expr::Relative(1),
            Expr::Group(vec![
                Expr::Relative(2),
                Expr::Relative(1),
                Expr::Relative(0),
            ]),
        ]),
    ))))));
    let mut expr = Expr::Group(vec![Expr::Group(vec![succ.clone()]), Expr::from(1)]);

    loop {
        println!("{}", expr.format(0));

        let betas = expr.find_beta_reductions();
        eprintln!("{betas:?}");

        if betas.is_empty() {
            break;
        }

        let mut max_i = 0;
        let mut last_score = 0;
        for (i, (score, _)) in betas.iter().enumerate() {
            if *score > last_score {
                last_score = *score;
                max_i = i;
            }
        }

        expr.beta_reduction_at(&betas[max_i].1);
    }

    expr.simplify();

    assert_eq!(expr, Expr::from(2))
}

#[test]
fn add() {
    // n:f:x:f (n f x) is succ
    // m:n:(m(n:f:x:(f(nfx)))n
    // a:b:(a(c:d:e:(d(cde)))b

    let succ = Expr::Def(Box::new(Expr::Def(Box::new(Expr::Def(Box::new(
        Expr::Group(vec![
            Expr::Relative(1),
            Expr::Group(vec![
                Expr::Relative(2),
                Expr::Relative(1),
                Expr::Relative(0),
            ]),
        ]),
    ))))));

    let add = Expr::Def(Box::new(Expr::Def(Box::new(Expr::Group(vec![
        Expr::Group(vec![Expr::Relative(1), Expr::Group(vec![succ.clone()])]),
        Expr::Relative(0),
    ])))));

    let mut expr = Expr::Group(vec![
        Expr::Group(vec![add.clone()]),
        Expr::from(2),
        Expr::from(2),
    ]);

    loop {
        println!("{}", expr.format(0));

        let betas = expr.find_beta_reductions();
        eprintln!("{betas:?}");

        if betas.is_empty() {
            break;
        }

        let mut max_i = 0;
        let mut last_score = 0;
        for (i, (score, _)) in betas.iter().enumerate() {
            if *score > last_score {
                last_score = *score;
                max_i = i;
            }
        }

        expr.beta_reduction_at(&betas[max_i].1);
    }

    expr.simplify();

    println!("F: {}", expr.format(0));

    assert_eq!(expr, Expr::parse("(f:x:(f(f(f(f x)))))").unwrap(),)
}
