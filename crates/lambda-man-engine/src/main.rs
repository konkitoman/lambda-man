use lambda_man_engine::Expr;

// add 1 2
// (a:b:(a c:d:e:(d(c d e))b) (a:b:(a b)) (a:b:(a(a b))))
//
// add (add 1 2) 2
// (a:b:(a c:d:e:(d(c d e))b) (a:b:(a c:d:e:(d(c d e))b) (a:b:(a b)) (a:b:(a(a b)))) (a:b:(a (a b))))
//
// sub 10 6
// ((m:n:(n(n:f:x:(n(g:h:(h(g f)))(u:x)(u:u)))m)) (f:x:(f(f(f(f(f(f(f(f(f(f x))))))))))) (f:x:(f(f(f(f(f(f x))))))))
//
// sub (fst (pair 5 1)) (snd (pair 5 1))
// ((m:n:(n(n:f:x:(n(g:h:(h(g f)))(u:x)(u:u)))m))((p:(p(x:y:x)))((x:y:f:(f x y))(f:x:(f(f(f(f(f x))))))(f:x:(f x))))((p:(p(x:y:y)))((x:y:f:(f x y))(f:x:(f(f(f(f(f x))))))(f:x:(f x)))))
fn main() {
    let stdin = std::io::stdin();
    loop {
        let mut line = String::default();
        print!("> ");
        _ = std::io::Write::flush(&mut std::io::stdout());
        if stdin.read_line(&mut line).is_err() {
            break;
        }

        if let Some(mut expr) = Expr::parse(line.trim()) {
            expr.simplify();
            println!("=={}", expr.format(0));

            loop {
                expr.simplify();
                let betas = expr.find_beta_reductions();

                for (score, at) in betas.iter() {
                    println!("\t{score}: {}", expr.format_highlightd(0, at, "31"))
                }

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

                println!(" {}", expr.format_highlightd(0, &betas[max_i].1, "31"));

                expr.beta_reduction_at(&betas[max_i].1);

                println!("={}", expr.format_highlightd(0, &betas[max_i].1, "32"));
            }

            expr.simplify();
            println!("={}", expr.format(0));
        } else {
            eprintln!("Cannot parse");
        }
    }
}
