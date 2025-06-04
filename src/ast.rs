#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let { name: String, expr: Expr },
    Return(Expr),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Var(String),
    Literal(i32),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expr::Var(name) => write!(f, "{}", name),
            Expr::Literal(n) => write!(f, "{}", n),
            Expr::Add(l, r) => write!(f, "({} + {})", l, r),
            Expr::Mul(l, r) => write!(f, "({} * {})", l, r),
        }
    }
}
