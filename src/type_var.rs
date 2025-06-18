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
    Any,
    Integer(usize),
    String(),
    Call(Place, Vec<TypeVar>, Vec<TypeVar>),
    BinOp(Place),
    None,
    Function(Place, Vec<TypeVar>, Vec<TypeVar>),
    Union(Vec<TypeVar>),
    Var(Place), // placeholder for unknown
}

impl TypeVar {
    /// Check if types are allowed
    /// Returns True when the conditions are OK
    /// eg. Int and Any would return `true`
    pub fn type_check(&self, other: &TypeVar) -> bool {
        match (self, other) {
            (TypeVar::Any, _) | (_, TypeVar::Any) => true,
            (TypeVar::Union(_left_tys), TypeVar::Union(_right_tys)) => todo!(),
            (TypeVar::Union(tys), x) | (x, TypeVar::Union(tys)) => tys.contains(x),
            (l, r) => std::mem::discriminant(l) == std::mem::discriminant(r),
        }
    }
}

impl std::fmt::Display for TypeVar {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Any => write!(f, "Any()"),
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
            Self::Function(p, param, ret) => {
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
                write!(f, "Function({}, [{}] -> [{}])", p, params_str, return_str)
            }
            Self::Union(v) => {
                let vals = v
                    .iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "Union({})", vals)
            }
            Self::BinOp(p) => write!(f, "BinOp({})", p),
            Self::Var(p) => write!(f, "Var({})", p),
            Self::String() => write!(f, "String()"),
            Self::None => write!(f, "None"),
        }
    }
}
