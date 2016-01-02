extern crate syntax;

use self::syntax::ptr::{B};
use self::syntax::ast::*;
use self::syntax::parse::parser::{Parser};
use self::syntax::visit::*;
use self::syntax::visitor_impl::{TypeChecker};

//#[test]
//fn test_parse_and_typecheck_var_decl(){
//    let mut p = Parser::new("let var a : int := 1 in end".to_string());
//    let b = p.run().unwrap();
//    let mut v = TypeChecker::new();
//    v.visit_expr(&*b.expr.unwrap());
//    assert_eq!(v.ty, TType::TInt32);
//}
