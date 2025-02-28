use crate::common::{FQName, Identifier, OrderClause, RelationElement, Span};
use itertools::Itertools;
use std::fmt::{Display, Formatter};

/// data for select statements
#[derive(PartialEq, Debug, Clone)]
pub struct Select {
    /// if true DISTINCT results
    pub distinct: bool,
    /// if true JSON reslts
    pub json: bool,
    /// The table name.
    pub table_name: FQName,
    /// the list of elements to select.
    pub columns: Vec<SelectElement>,
    /// the where clause
    pub where_clause: Vec<RelationElement>,
    /// the optional ordering
    pub order: Option<OrderClause>,
    /// the number of items to return
    pub limit: Option<i32>,
    /// if true ALLOW FILTERING is displayed
    pub filtering: bool,
}

impl Select {
    /// return the column names selected
    /// does not return functions.
    pub fn select_names(&self) -> Vec<String> {
        self.columns
            .iter()
            .filter_map(|e| {
                if let SelectElement::Column(named) = e {
                    Some(named.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    /// return the aliased column names.  If the column is not aliased the
    /// base column name is returned.
    /// does not return functions.
    pub fn select_alias(&self) -> Vec<Identifier> {
        self.columns
            .iter()
            .filter_map(|e| match e {
                SelectElement::Column(named) => {
                    Some(named.alias.clone().unwrap_or_else(|| named.name.clone()))
                }
                _ => None,
            })
            .collect()
    }
}

impl Display for Select {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SELECT {}{}{} FROM {}{}{}{}{}",
            if self.distinct { "DISTINCT " } else { "" },
            if self.json { "JSON " } else { "" },
            self.columns.iter().join(", "),
            self.table_name,
            if !self.where_clause.is_empty() {
                format!(" WHERE {}", self.where_clause.iter().join(" AND "))
            } else {
                "".to_string()
            },
            self.order
                .as_ref()
                .map_or("".to_string(), |x| format!(" ORDER BY {}", x)),
            self.limit
                .map_or("".to_string(), |x| format!(" LIMIT {}", x)),
            if self.filtering {
                " ALLOW FILTERING"
            } else {
                ""
            }
        )
    }
}

/// the selectable elements for a select statement
#[derive(PartialEq, Debug, Clone)]
pub enum SelectElement {
    /// All of the columns
    Star,
    /// a named column.  May have an alias specified.
    Column(Named),
    /// a named column.  May have an alias specified.
    Function(Named),
}

impl Display for SelectElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectElement::Star => write!(f, "*"),
            SelectElement::Column(named) | SelectElement::Function(named) => write!(f, "{}", named),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Named {
    pub name: Identifier,
    pub alias: Option<Identifier>,
}

/// the name an optional alias for a named item.
impl Named {
    pub fn new(name: &str, name_span: Span, alias: &str, alias_span: Span) -> Named {
        Named {
            name: Identifier::parse(name, name_span),
            alias: Some(Identifier::parse(alias, alias_span)),
        }
    }

    pub fn simple(name: &str, span: Span) -> Named {
        Named {
            name: Identifier::parse(name, span),
            alias: None,
        }
    }

    pub fn alias_or_name(&self) -> &Identifier {
        match &self.alias {
            None => &self.name,
            Some(alias) => alias,
        }
    }
}

impl Display for Named {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.alias {
            None => write!(f, "{}", self.name),
            Some(a) => write!(f, "{} AS {}", self.name, a),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::select::{Named, SelectElement, Span};

    #[test]
    fn test_select_element_display() {
        assert_eq!("*", SelectElement::Star.to_string());
        assert_eq!(
            "col",
            SelectElement::Column(Named::simple("col", Span::from("col"))).to_string()
        );
        assert_eq!(
            "func",
            SelectElement::Function(Named::simple("func", Span::from("func"))).to_string()
        );
        assert_eq!(
            "col AS alias",
            SelectElement::Column(Named::new(
                "col",
                Span::from("func"),
                "alias",
                Span::from("alias")
            ))
            .to_string()
        );
        assert_eq!(
            "func AS alias",
            SelectElement::Function(Named::new(
                "func",
                Span::from("func"),
                "alias",
                Span::from("alias")
            ))
            .to_string()
        );
    }
}
