use tree_sitter::Point;

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct Place {
    pub name: String,
    pub row: usize,
    pub column: usize,
}

impl Place {
    pub fn exp_from_ts_point(point: Point) -> Self {
        Place {
            name: "exp".to_owned(),
            row: point.row,
            column: point.column,
        }
    }
    pub fn from_ts_point(name: &str, point: Point) -> Self {
        Place {
            name: name.to_owned(),
            row: point.row,
            column: point.column,
        }
    }
}

impl std::fmt::Display for Place {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}@{},{}", self.name, self.row, self.column)
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub enum TypeVar {
    Any(),
    Integer(usize),
    String(),
    Call(Place, Vec<TypeVar>, Vec<TypeVar>),
    BinOp(Place),
    Var(Place), // placeholder for unknown
}

impl std::fmt::Display for TypeVar {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Any() => write!(f, "Any()"),
            Self::Integer(i) => write!(f, "Integer({})", i),
            Self::Call(p, param, ret) => {
                let params_str = param
                    .iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(",");
                let return_str = ret
                    .iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "Call({}, [{}] -> [{}])", p, params_str, return_str)
            }
            Self::BinOp(p) => write!(f, "BinOp({})", p),
            Self::Var(p) => write!(f, "Var({})", p),
            Self::String() => write!(f, "String()"),
        }
    }
}
