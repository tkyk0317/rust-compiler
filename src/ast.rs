use std::collections::HashMap;
use symbol::{Scope, Structure, Symbol, SymbolTable, Type};
use token::{Token, TokenInfo};

#[derive(Debug, Clone, PartialEq)]
pub enum AstType {
    Global(Vec<AstType>),
    FuncDef(Type, Structure, String, Box<AstType>, Box<AstType>),
    Statement(Vec<AstType>),
    While(Box<AstType>, Box<AstType>), // 条件式、ブロック部.
    Do(Box<AstType>, Box<AstType>),    // ブロック部、条件式.
    If(Box<AstType>, Box<AstType>, Box<Option<AstType>>), // 条件式、真ブロック、偽ブロック.
    For(
        Box<Option<AstType>>,
        Box<Option<AstType>>,
        Box<Option<AstType>>,
        Box<AstType>,
    ), // 初期条件、終了条件、更新部、ブロック部.
    Continue(),
    Break(),
    Return(Box<AstType>),
    Condition(Box<AstType>, Box<AstType>, Box<AstType>),
    LogicalAnd(Box<AstType>, Box<AstType>),
    LogicalOr(Box<AstType>, Box<AstType>),
    BitAnd(Box<AstType>, Box<AstType>),
    BitOr(Box<AstType>, Box<AstType>),
    BitXor(Box<AstType>, Box<AstType>),
    Equal(Box<AstType>, Box<AstType>),
    NotEqual(Box<AstType>, Box<AstType>),
    LessThan(Box<AstType>, Box<AstType>),
    GreaterThan(Box<AstType>, Box<AstType>),
    LessThanEqual(Box<AstType>, Box<AstType>),
    GreaterThanEqual(Box<AstType>, Box<AstType>),
    Plus(Box<AstType>, Box<AstType>),
    Minus(Box<AstType>, Box<AstType>),
    LeftShift(Box<AstType>, Box<AstType>),
    RightShift(Box<AstType>, Box<AstType>),
    Multiple(Box<AstType>, Box<AstType>),
    Division(Box<AstType>, Box<AstType>),
    Remainder(Box<AstType>, Box<AstType>),
    UnPlus(Box<AstType>),
    UnMinus(Box<AstType>),
    Not(Box<AstType>),
    BitReverse(Box<AstType>),
    Assign(Box<AstType>, Box<AstType>),
    Factor(i64),
    Variable(Type, Structure, String),
    FuncCall(Box<AstType>, Box<AstType>),
    Argment(Vec<AstType>),
    Address(Box<AstType>),
    Indirect(Box<AstType>),
    PreInc(Box<AstType>),
    PreDec(Box<AstType>),
    PostInc(Box<AstType>),
    PostDec(Box<AstType>),
    StringLiteral(String, usize),
    PlusAssign(Box<AstType>, Box<AstType>),
    MinusAssign(Box<AstType>, Box<AstType>),
    MultipleAssign(Box<AstType>, Box<AstType>),
    DivisionAssign(Box<AstType>, Box<AstType>),
    RemainderAssign(Box<AstType>, Box<AstType>),
    SizeOf(usize),
    Struct(Box<AstType>, Vec<AstType>),
}

impl AstType {
    // 式判定.
    pub fn is_expr(&self) -> bool {
        match self {
            AstType::If(_, _, _)
            | AstType::For(_, _, _, _)
            | AstType::Do(_, _)
            | AstType::Continue()
            | AstType::Break()
            | AstType::Return(_)
            | AstType::While(_, _) => false,
            _ => true,
        }
    }
}

#[derive(Debug)]
pub struct AstGen<'a> {
    tokens: &'a [TokenInfo], // トークン配列.
    current_pos: usize,         // 現在読み取り位置.
    str_count: usize,           // 文字列リテラル位置
    f_sym: HashMap<String, (Type, Structure)>,
    cur_scope: Scope,
    sym_table: SymbolTable,
}

#[derive(Debug)]
pub struct AstTree {
    pub tree: Vec<AstType>, // 抽象構文木.
}

// 抽象構文木.
impl AstTree {
    // コンストラクタ.
    fn new(t: Vec<AstType>) -> Self {
        AstTree { tree: t }
    }

    // 抽象構文木取得.
    pub fn get_tree(&self) -> &Vec<AstType> {
        &self.tree
    }
}

// 抽象構文木をトークン列から作成する
impl<'a> AstGen<'a> {
    // コンストラクタ.
    pub fn new(t: &'a [TokenInfo]) -> AstGen<'a> {
        AstGen {
            current_pos: 0,
            str_count: 0,
            tokens: t,
            f_sym: HashMap::new(),
            cur_scope: Scope::Global,
            sym_table: SymbolTable::new(),
        }
    }

    // シンボルテーブル取得
    pub fn get_symbol(&self) -> &SymbolTable {
        &self.sym_table
    }

    // トークン列を受け取り、抽象構文木を返す.
    pub fn parse(&mut self) -> AstTree {
        // グローバル変数
        let g = self.global_var(vec![]);
        let mut s = if g.is_empty() {
            vec![]
        } else {
            vec![AstType::Global(g)]
        };

        // 関数定義
        while self.next().get_token_type() != Token::End {
            let expr = self.func_def();
            s.push(expr);
        }
        AstTree::new(s)
    }

    // スコープ切り替え
    fn switch_scope(&mut self, scope: Scope) {
        self.cur_scope = scope;
    }

    // global variable
    fn global_var(&mut self, acc: Vec<AstType>) -> Vec<AstType> {
        self.switch_scope(Scope::Global);

        // タイプを判断する為、先読み
        let (_t, s) = self.generate_type();
        let token = self.next_consume();
        let paren = self.next();

        // 先読み分を戻る
        self.back(2);
        match token.get_token_type() {
            // 変数定義
            Token::Variable if s != Structure::Struct && Token::LeftParen != paren.get_token_type() => {
                // グローバル変数
                let var = self.assign();
                self.must_next(
                    Token::SemiColon,
                    "ast.rs(global_var): Not exists semi-colon",
                );

                let mut vars = acc;
                vars.push(var);
                self.global_var(vars)
            },
            // 構造体定義
            Token::Variable if s == Structure::Struct  => {
                // Token::Structまでもどっているので一つSKIP
                self.consume();

                // 構造体定義作成
                let mut vars = acc;
                vars.push(self.struct_def_or_var());
                self.global_var(vars)
            },
            _ => acc,
        }
    }

    // func def.
    fn func_def(&mut self) -> AstType {
        // 型を取得.
        let (t, s) = self.generate_type();

        // 関数定義から始まらないとだめ（関数の中に様々な処理が入っている）.
        let token = self.next_consume();
        match token.get_token_type() {
            Token::Variable => {
                self.switch_scope(Scope::Local(token.get_token_value()));

                // 既に同じシンボルが登録されていればエラー.
                if self.search_symbol(&Scope::Func, &token.get_token_value()).is_some() {
                    panic!("{} {}: already define {}", file!(), line!(), token.get_token_value());
                }

                // 関数シンボルを登録.
                self.sym_table.register_sym(Symbol::new(
                    Scope::Func,
                    token.get_token_value(),
                    t.clone(),
                    s.clone(),
                ));

                AstType::FuncDef(
                    t,
                    s,
                    token.get_token_value(),
                    Box::new(self.func_args()),
                    Box::new(self.statement()),
                )
            }
            _ => panic!(
                "{} {}: Not Exists Function def {:?}",
                file!(),
                line!(),
                token
            ),
        }
    }

    // typeトークンチェック
    fn is_type_token(&mut self) -> bool {
        match self.next().get_token_type() {
            Token::Int | Token::IntPointer | Token::Char | Token::CharPointer => true,
            _ => false,
        }
    }

    // type/struct judge
    fn generate_type(&mut self) -> (Type, Structure) {
        let token = self.next_consume();
        match token.get_token_type() {
            Token::Int => (Type::Int, Structure::Identifier),
            Token::IntPointer => (Type::Int, Structure::Pointer),
            Token::Char => (Type::Char, Structure::Identifier),
            Token::CharPointer => (Type::Char, Structure::Pointer),
            Token::Struct => {
                // 構造体の定義名を取得
                let name = self.next();
                (Type::Struct(name.get_token_value()), Structure::Struct)
            }
            _ => (Type::Unknown("unknown type".to_string()), Structure::Unknown),
        }
    }

    // func argment.
    fn func_args(&mut self) -> AstType {
        let token = self.next_consume();
        match token.get_token_type() {
            Token::LeftParen => {
                // 引数を処理.
                let args = AstType::Argment(self.recur_func_args(vec![]));

                // 閉じ括弧.
                self.must_next(Token::RightParen, "ast.rs(func_arg): Not Exists RightParen");
                args
            }
            _ => panic!("{} {}: Not Exists LeftParen {:?}", file!(), line!(), token),
        }
    }

    // recur func argment.
    fn recur_func_args(&mut self, a: Vec<AstType>) -> Vec<AstType> {
        // 型が定義されていれば、引数として評価.
        if !self.is_type_token() {
            return a;
        }

        // 引数を評価
        let mut args = a;
        args.push(self.assign());

        // カンマがあれば引き続き.
        match self.next().get_token_type() {
            Token::Comma => {
                self.consume();
                self.recur_func_args(args)
            }
            _ => args,
        }
    }

    // statement.
    fn statement(&mut self) -> AstType {
        AstType::Statement(self.sub_statement(&[]))
    }

    // sub statement.
    fn sub_statement(&mut self, expr: &[AstType]) -> Vec<AstType> {
        // トークンがなくなるまで、構文木生成.
        let mut stmt = expr.to_owned();
        let token = self.next_consume();
        match token.get_token_type() {
            Token::If => {
                stmt.push(self.statement_if());
                self.sub_statement(&stmt)
            }
            Token::While => {
                stmt.push(self.statement_while());
                self.sub_statement(&stmt)
            }
            Token::For => {
                stmt.push(self.statement_for());
                self.sub_statement(&stmt)
            }
            Token::Do => {
                stmt.push(self.statement_do());
                self.sub_statement(&stmt)
            }
            Token::Continue => {
                stmt.push(self.statement_continue());
                self.sub_statement(&stmt)
            }
            Token::Break => {
                stmt.push(self.statement_break());
                self.sub_statement(&stmt)
            }
            Token::LeftBrace => self.sub_statement(&stmt),
            Token::SemiColon => self.sub_statement(&stmt),
            Token::RightBrace => stmt,
            Token::Comma => {
                // 前の変数の型を考慮
                let var = self.continue_variable_define(&stmt);
                stmt.push(var);
                self.sub_statement(&stmt)
            }
            _ => {
                self.back(1);
                stmt.push(self.expression());
                self.sub_statement(&stmt)
            }
        }
    }

    // continue variable
    fn continue_variable_define(&mut self, stmt: &[AstType]) -> AstType {
        let last = stmt.last();
        match last {
            Some(ref s) => match s {
                AstType::Variable(ref t, ref s, ref _n) => match t {
                    Type::Int if s == &Structure::Identifier => self.factor_int(),
                    Type::Char if s == &Structure::Identifier => self.factor_char(),
                    Type::Int if s == &Structure::Pointer => {
                        self.variable(Type::Int, Structure::Pointer)
                    }
                    Type::Char if s == &Structure::Pointer => {
                        self.variable(Type::Char, Structure::Pointer)
                    }
                    _ => panic!("{} {}: Not Support Type {:?}", file!(), line!(), t),
                },
                _ => panic!("{} {}: Not Support Ast {:?}", file!(), line!(), s),
            },
            None => panic!("{} {}: Not exists Variable", file!(), line!()),
        }
    }

    // if statement.
    //
    // ブロック部が一行の場合、asm部が期待しているAstType::Statementでexpression結果を包む
    fn statement_if(&mut self) -> AstType {
        self.must_next(
            Token::LeftParen,
            "ast.rs(statement_if): Not Exists LeftParen",
        );

        // 条件式を解析.
        let condition = self.assign();
        self.must_next(
            Token::RightParen,
            "ast.rs(statement_if): Not Exists RightParen",
        );

        // ifブロック内を解析.
        let stmt = match self.next().get_token_type() {
            Token::LeftBrace => self.statement(),
            _ => {
                let expr = AstType::Statement(vec![self.expression()]);
                self.must_next(
                    Token::SemiColon,
                    "ast.rs(statement_if): Not Exists SemiColon",
                );
                expr
            }
        };

        // else部分解析.
        match self.next().get_token_type() {
            Token::Else => {
                self.consume();
                let else_stmt = match self.next().get_token_type() {
                    Token::LeftBrace => self.statement(),
                    _ => {
                        let expr = AstType::Statement(vec![self.expression()]);
                        self.must_next(
                            Token::SemiColon,
                            "ast.rs(statement_if): Not Exists SemiColon",
                        );
                        expr
                    }
                };
                AstType::If(
                    Box::new(condition),
                    Box::new(stmt),
                    Box::new(Some(else_stmt)),
                )
            }
            _ => AstType::If(Box::new(condition), Box::new(stmt), Box::new(None)),
        }
    }

    // while statement.
    fn statement_while(&mut self) -> AstType {
        self.must_next(
            Token::LeftParen,
            "ast.rs(statement_while): Not Exists LeftParen",
        );

        // 条件式を解析.
        let condition = self.assign();
        self.must_next(
            Token::RightParen,
            "ast.rs(statement_while): Not Exists RightParen",
        );

        AstType::While(Box::new(condition), Box::new(self.statement()))
    }

    // do-while statement.
    fn statement_do(&mut self) -> AstType {
        // ブロック部.
        let stmt = self.statement();
        self.must_next(Token::While, "ast.rs(statement_do): Not Exists while token");

        // 条件式を解析.
        self.must_next(
            Token::LeftParen,
            "ast.rs(statement_do): Not Exists LeftParen",
        );
        let condition = self.assign();
        self.must_next(
            Token::RightParen,
            "ast.rs(statement_while): Not Exists RightParen",
        );

        AstType::Do(Box::new(stmt), Box::new(condition))
    }

    // for statement.
    fn statement_for(&mut self) -> AstType {
        self.must_next(
            Token::LeftParen,
            "ast.rs(statement_for): Not Exists LeftParen",
        );

        // 各種条件を解析.
        let begin = match self.next().get_token_type() {
            Token::SemiColon => None,
            _ => Some(self.assign()),
        };
        self.must_next(
            Token::SemiColon,
            "ast.rs(statement_for): Not Exists Semicolon",
        );

        let condition = match self.next().get_token_type() {
            Token::SemiColon => None,
            _ => Some(self.assign()),
        };
        self.must_next(
            Token::SemiColon,
            "ast.rs(statement_for): Not Exists Semicolon",
        );

        let end = match self.next().get_token_type() {
            Token::RightParen => None,
            _ => Some(self.assign()),
        };
        self.must_next(
            Token::RightParen,
            "ast.rs(statement_for): Not Exists RightParen",
        );

        AstType::For(
            Box::new(begin),
            Box::new(condition),
            Box::new(end),
            Box::new(self.statement()),
        )
    }

    // continue statement.
    fn statement_continue(&mut self) -> AstType {
        AstType::Continue()
    }

    // break statement.
    fn statement_break(&mut self) -> AstType {
        AstType::Break()
    }

    // return statement.
    fn statement_return(&mut self) -> AstType {
        let expr = self.assign();
        AstType::Return(Box::new(expr))
    }

    // expression.
    fn expression(&mut self) -> AstType {
        match self.next().get_token_type() {
            Token::Return => {
                self.consume();
                self.statement_return()
            }
            _ => self.assign(),
        }
    }

    // assign.
    fn assign(&mut self) -> AstType {
        let token = self.next_consume();
        let next_token = self.next();

        // Variableトークンへ位置を戻す
        self.back(1);
        match token.get_token_type() {
            Token::Variable if Token::Assign == next_token.get_token_type() => {
                let var = self.factor();
                self.consume();  // Assignトークン消費
                AstType::Assign(Box::new(var), Box::new(self.condition()))
            }
            Token::Variable if Token::PlusAssign == next_token.get_token_type() => {
                let var = self.factor();
                self.consume();  // Assignトークン消費
                AstType::PlusAssign(Box::new(var), Box::new(self.condition()))
            }
            Token::Variable if Token::MinusAssign == next_token.get_token_type() => {
                let var = self.factor();
                self.consume();  // Assignトークン消費
                AstType::MinusAssign(Box::new(var), Box::new(self.condition()))
             }
            Token::Variable if Token::MultipleAssign == next_token.get_token_type() => {
                let var = self.factor();
                self.consume();  // Assignトークン消費
                AstType::MultipleAssign(Box::new(var), Box::new(self.condition()))
            }
            Token::Variable if Token::DivisionAssign == next_token.get_token_type() => {
                let var = self.factor();
                self.consume();  // Assignトークン消費
                AstType::DivisionAssign(Box::new(var), Box::new(self.condition()))
            }
            Token::Variable if Token::RemainderAssign == next_token.get_token_type() => {
                let var = self.factor();
                self.consume();  // Assignトークン消費
                AstType::RemainderAssign(Box::new(var), Box::new(self.condition()))
             }
             _ => self.condition(),
        }
    }

    // func call.
    fn call_func(&mut self, acc: AstType) -> AstType {
        let token = self.next_consume();
        match token.get_token_type() {
            Token::LeftParen => {
                let call_func = AstType::FuncCall(
                    Box::new(acc),
                    Box::new(self.argment(AstType::Argment(vec![]))),
                );
                self.must_next(
                    Token::RightParen,
                    "ast.rs(call_func): Not exists RightParen",
                );
                call_func
            }
            _ => panic!("{} {}: Not exists LeftParen: {:?}", file!(), line!(), token),
        }
    }

    // sub argment
    fn sub_argment(&mut self, acc: AstType) -> AstType {
        match acc {
            AstType::Argment(a) => {
                let mut args = a;
                args.push(self.assign());

                // カンマがあれば引き続き、引数とみなす.
                if Token::Comma == self.next().get_token_type() {
                    self.next_consume();
                    self.argment(AstType::Argment(args))
                } else {
                    AstType::Argment(args)
                }
            }
            _ => panic!("{} {}: Not Support AstType {:?}", file!(), line!(), acc),
        }
    }

    // argment.
    fn argment(&mut self, acc: AstType) -> AstType {
        // 右括弧が表れるまで、引数とみなす
        let token = self.next();
        match token.get_token_type() {
            Token::RightParen => acc,
            _ => self.sub_argment(acc),
        }
    }

    // condition.
    fn condition(&mut self) -> AstType {
        let left = self.logical();
        self.sub_condition(left)
    }

    // sub condition.
    fn sub_condition(&mut self, acc: AstType) -> AstType {
        let ope_type = self.next().get_token_type();
        match ope_type {
            Token::Question => {
                self.consume();
                let middle = self.logical();

                // コロンがない場合、終了.
                self.must_next(Token::Colon, "ast.rs(sub_condition): Not exists Colon");

                let right = self.logical();
                let tree = AstType::Condition(Box::new(acc), Box::new(middle), Box::new(right));
                self.sub_condition(tree)
            }
            _ => acc,
        }
    }

    // logical.
    fn logical(&mut self) -> AstType {
        let left = self.bit_operator();
        self.sub_logical(left)
    }

    // sub logical.
    fn sub_logical(&mut self, acc: AstType) -> AstType {
        let create = |ope: Token, left, right| match ope {
            Token::LogicalAnd => AstType::LogicalAnd(Box::new(left), Box::new(right)),
            Token::Assign => AstType::Assign(Box::new(left), Box::new(right)),
            _ => AstType::LogicalOr(Box::new(left), Box::new(right)),
        };

        let ope_type = self.next().get_token_type();
        match ope_type {
            Token::LogicalAnd | Token::LogicalOr | Token::Assign => {
                self.consume();
                let right = self.bit_operator();
                self.sub_logical(create(ope_type, acc, right))
            }
            _ => acc,
        }
    }

    // bit operator.
    fn bit_operator(&mut self) -> AstType {
        let left = self.relation();
        self.sub_bit_operator(left)
    }

    // sub bit operator.
    fn sub_bit_operator(&mut self, acc: AstType) -> AstType {
        let create = |ope, left, right| match ope {
            Token::BitOr => AstType::BitOr(Box::new(left), Box::new(right)),
            Token::And => AstType::BitAnd(Box::new(left), Box::new(right)),
            Token::BitXor => AstType::BitXor(Box::new(left), Box::new(right)),
            _ => panic!("{} {}: Not Support Token {:?}", file!(), line!(), ope),
        };

        let token = self.next();
        match token.get_token_type() {
            Token::BitOr | Token::And | Token::BitXor => {
                self.consume();
                let right = self.relation();
                self.sub_bit_operator(create(token.get_token_type(), acc, right))
            }
            _ => acc,
        }
    }

    // relation.
    fn relation(&mut self) -> AstType {
        let left = self.shift();
        self.sub_relation(left)
    }

    // sub relation.
    fn sub_relation(&mut self, acc: AstType) -> AstType {
        let create = |ope: Token, left, right| match ope {
            Token::Equal => AstType::Equal(Box::new(left), Box::new(right)),
            Token::NotEqual => AstType::NotEqual(Box::new(left), Box::new(right)),
            Token::LessThan => AstType::LessThan(Box::new(left), Box::new(right)),
            Token::GreaterThan => AstType::GreaterThan(Box::new(left), Box::new(right)),
            Token::LessThanEqual => AstType::LessThanEqual(Box::new(left), Box::new(right)),
            Token::GreaterThanEqual => AstType::GreaterThanEqual(Box::new(left), Box::new(right)),
            _ => panic!("{} {}: Not Support Token Type {:?}", file!(), line!(), ope),
        };

        let ope_type = self.next().get_token_type();
        match ope_type {
            Token::Equal
            | Token::NotEqual
            | Token::LessThan
            | Token::LessThanEqual
            | Token::GreaterThan
            | Token::GreaterThanEqual => {
                self.consume();
                let right = self.shift();
                self.sub_relation(create(ope_type, acc, right))
            }
            _ => acc,
        }
    }

    // shift operation.
    fn shift(&mut self) -> AstType {
        let left = self.expr();
        self.sub_shift(left)
    }

    fn sub_shift(&mut self, acc: AstType) -> AstType {
        let create = |ope: Token, left, right| match ope {
            Token::LeftShift => AstType::LeftShift(Box::new(left), Box::new(right)),
            Token::RightShift => AstType::RightShift(Box::new(left), Box::new(right)),
            _ => panic!("{} {}: Not Support Token {:?}", file!(), line!(), ope),
        };

        let token = self.next();
        match token.get_token_type() {
            Token::LeftShift | Token::RightShift => {
                self.consume();
                let right = self.expr();
                self.sub_shift(create(token.get_token_type(), acc, right))
            }
            _ => acc,
        }
    }

    // expression
    fn expr(&mut self) -> AstType {
        let left = self.term();
        self.expr_add_sub(left)
    }

    // add or sub expression.
    fn expr_add_sub(&mut self, acc: AstType) -> AstType {
        let create = |ope, left, right| match ope {
            Token::Plus => AstType::Plus(Box::new(left), Box::new(right)),
            _ => AstType::Minus(Box::new(left), Box::new(right)),
        };

        let ope = self.next();
        match ope.get_token_type() {
            Token::Plus | Token::Minus => {
                self.consume();
                let right = self.term();
                self.expr_add_sub(create(ope.get_token_type(), acc, right))
            }
            _ => acc,
        }
    }

    // term.
    fn term(&mut self) -> AstType {
        let left = self.factor();
        self.term_multi_div(left)
    }

    // multiple and division term.
    fn term_multi_div(&mut self, acc: AstType) -> AstType {
        let create = |ope, left, right| match ope {
            Token::Multi => AstType::Multiple(Box::new(left), Box::new(right)),
            Token::Division => AstType::Division(Box::new(left), Box::new(right)),
            _ => AstType::Remainder(Box::new(left), Box::new(right)),
        };

        let ope = self.next();
        match ope.get_token_type() {
            Token::Multi | Token::Division | Token::Remainder => {
                self.consume();
                let right = self.factor();
                self.term_multi_div(create(ope.get_token_type(), acc, right))
            }
            _ => acc,
        }
    }

    // factor.
    fn factor(&mut self) -> AstType {
        let token = self.next_consume();
        match token.get_token_type() {
            Token::Inc => AstType::PreInc(Box::new(self.factor())),
            Token::Dec => AstType::PreDec(Box::new(self.factor())),
            Token::Plus => AstType::UnPlus(Box::new(self.factor())),
            Token::Minus => AstType::UnMinus(Box::new(self.factor())),
            Token::Not => AstType::Not(Box::new(self.factor())),
            Token::BitReverse => AstType::BitReverse(Box::new(self.factor())),
            Token::SizeOf => self.factor_sizeof(),
            Token::IntPointer => self.variable(Type::Int, Structure::Pointer),
            Token::CharPointer => self.variable(Type::Char, Structure::Pointer),
            Token::And => AstType::Address(Box::new(self.factor())),
            Token::Multi => AstType::Indirect(Box::new(self.factor())),
            Token::Number => self.number(token),
            Token::Int => self.factor_int(),
            Token::Char => self.factor_char(),
            Token::StringLiteral => self.string_literal(token),
            Token::Struct => self.struct_def_or_var(),
            Token::Variable => {
                // variable位置へ
                self.back(1);
                self.factor_variable(&token)
            }
            Token::LeftParen => {
                let tree = self.assign();
                self.must_next(Token::RightParen, "ast.rs(factor): Not exists RightParen");
                tree
            }
            _ => panic!("{} {}: failed in factor {:?}", file!(), line!(), token),
        }
    }

    // 構造体定義、宣言作成
    fn struct_def_or_var(&mut self) -> AstType {
        let def_name = self.next_consume();
        let token = self.next_consume();
        match token.get_token_type() {
            Token::LeftBrace => self.struct_def(def_name),
            Token::Variable => self.struct_variable(def_name, token),
            _ => panic!("{} {}: failed in struct_def_or_var {:?} {:?}", file!(), line!(), def_name, token),
        }
    }

    /// 構造体定義作成
    ///
    /// 構造体定義でシンボル登録し、ASTを返却
    fn struct_def(&mut self, def_name: &TokenInfo) -> AstType {
        // 右波括弧が出てくるまで、メンバー定義
        let mut right_brace = self.next();
        let mut members = vec![];
        let mut syms = vec![];
        loop {
            match right_brace.get_token_type() {
                Token::RightBrace => {
                    self.consume();
                    self.must_next(
                        Token::SemiColon, "ast.rs(struct_def_or_var): Not exists SemiColon"
                    );
                    break;
                }
                _ => {
                    // 構造体に所属しているメンバーをシンボルに登録
                    let member = self.assign();
                    let mem_sym = match member {
                        AstType::Variable(ref t, ref st, ref mem_name) => {
                            Symbol::new(self.cur_scope.clone(), mem_name.clone(), t.clone(), st.clone())
                        }
                        _ => panic!("not find variable")
                    };
                    members.push(member);
                    syms.push(mem_sym);

                    self.must_next(
                        Token::SemiColon, "ast.rs(struct_def_or_var): Not exists SemiColon"
                    );
                }
            };
            right_brace = self.next();
        }

        // シンボルテーブルへ構造体定義を保存（未登録の場合）.
        if self.search_symbol(&self.cur_scope, &def_name.get_token_value()).is_none() {
            let mut sym = Symbol::new(
                self.cur_scope.clone(),
                def_name.get_token_value(), // 構造体定義名で作成
                Type::Struct(def_name.get_token_value()),
                Structure::Struct,
            );
            // 構造体メンバーを登録し、シンボル保存
            sym.regist_mem(syms);
            self.sym_table.register_sym(sym);
        }

        AstType::Struct(
            Box::new( AstType::Variable(
                    Type::Struct(def_name.get_token_value()),
                    Structure::Struct,
                    def_name.get_token_value()
            )),
            members
        )
    }

    /// 構造体変数作成
    ///
    /// 構造体変数名でシンボルに登録し、ASTを返却
    fn struct_variable(&mut self, def_name: &TokenInfo, name: &TokenInfo) -> AstType {
        // 定義がシンボルテーブルに保存されているので、それを元にシンボル保存
        if let Some(s) = self.search_symbol(&self.cur_scope, &def_name.get_token_value()) {
            let mut sym = Symbol::new(
                self.cur_scope.clone(),
                name.get_token_value(), // 構造体変数名で作成
                Type::Struct(def_name.get_token_value()),
                Structure::Struct,
            );

            // 構造体定義よりメンバーを設定し、シンボル登録
            sym.regist_mem(s.members);
            self.sym_table.register_sym(sym);
        }

        AstType::Variable(
            Type::Struct(def_name.get_token_value()), Structure::Struct, name.get_token_value()
        )
    }

    // 文字列作成
    fn string_literal(&mut self, token: &TokenInfo) -> AstType {
        let count = self.str_count;
        self.str_count += 1;
        AstType::StringLiteral(token.get_token_value(), count)
    }

    // variable型の作成
    fn factor_variable(&mut self, token: &TokenInfo) -> AstType {
        // 変数シンボルサーチ
        match self.search_symbol(&self.cur_scope, &token.get_token_value()) {
            Some(ref sym) => {
                // 後置演算子判定
                let var = self.variable(sym.t.clone(), sym.strt.clone());
                match self.next().get_token_type() {
                    Token::Inc => {
                        self.consume();
                        AstType::PostInc(Box::new(var))
                    }
                    Token::Dec => {
                        self.consume();
                        AstType::PostDec(Box::new(var))
                    }
                    _ => var,
                }
            }
            None => {
                // 関数シンボルサーチ
                match self.search_symbol(&Scope::Func, &token.get_token_value()) {
                    Some(s) => {
                        let f_sym = self.variable_func(s.t.clone(), s.strt);
                        self.call_func(f_sym)
                    }
                    _ => panic!("{} {}: cannot define {:?}", file!(), line!(), token),
                }
            }
        }
    }

    // int型要素の作成
    fn factor_int(&mut self) -> AstType {
        // 配列かどうか決定する為に、一文字読み飛ばして、後で戻る
        let _ = self.next_consume();
        let token = self.next();
        self.back(1);
        match token.get_token_type() {
            Token::LeftBracket => self.variable_array(Type::Int),
            _ => self.variable(Type::Int, Structure::Identifier),
        }
    }

    // char型要素の作成
    fn factor_char(&mut self) -> AstType {
        // 配列かどうか決定する為に、一文字読み飛ばして、後で戻る
        let _ = self.next_consume();
        let token = self.next();
        self.back(1);
        match token.get_token_type() {
            Token::LeftBracket => self.variable_array(Type::Char),
            _ => self.variable(Type::Char, Structure::Identifier),
        }
    }

    // array index
    fn array_index(&mut self, s: &Structure) -> AstType {
        self.consume();
        let index = self.expression();
        self.must_next(
            Token::RightBracket,
            "ast.rs(variable): Not exists RightBracket",
        );
        // 多次元配列か？
        match self.next().get_token_type() {
            // 最初のインデックス分のオフセットを算出
            Token::LeftBracket => {
                let (count, tails) = match s {
                    Structure::Array(v) => (v[1] as i64, v.split_first().unwrap().1.to_vec()),
                    _ => panic!("ast.rs(array_index): cannot support structure {:?}", s),
                };
                let offset = AstType::Multiple(
                    Box::new(index), Box::new(AstType::Factor(count))
                );
                AstType::Plus(
                    Box::new(offset),
                    Box::new(self.array_index(&Structure::Array(tails))),
                )
            }
            _ => index,
        }
    }

    // variable.
    fn variable(&mut self, t: Type, s: Structure) -> AstType {
        let token = self.next_consume();
        let next = self.next();
        match token.get_token_type() {
            Token::Variable if Token::LeftBracket == next.get_token_type() => {
                // ポインタと同じようにアクセスするため、Indirectでくるむ
                let index = self.array_index(&s);
                AstType::Indirect(Box::new(AstType::Plus(
                    Box::new(AstType::Variable(t, s, token.get_token_value())),
                    Box::new(index),
                )))
            }
            Token::Variable => {
                // シンボルテーブルへ保存（未登録の場合）.
                if self.search_symbol(&self.cur_scope, &token.get_token_value()).is_none() {
                    self.sym_table.register_sym(Symbol::new(
                            self.cur_scope.clone(),
                            token.get_token_value(),
                            t.clone(),
                            s.clone(),
                    ));
                }
                AstType::Variable(t, s, token.get_token_value())
            }
            _ => panic!("{} {}: not support token {:?}", file!(), line!(), token),
        }
    }

    // function name.
    fn variable_func(&mut self, t: Type, s: Structure) -> AstType {
        // 関数名は定義時に登録されている為、シンボルテーブルには追加しない
        let token = self.next_consume();
        match token.get_token_type() {
            Token::Variable => AstType::Variable(t, s, token.get_token_value()),
            _ => panic!("{} {}: not support token {:?}", file!(), line!(), token),
        }
    }

    // array num count
    fn array_size(&mut self, size: Vec<usize>) -> Vec<usize> {
        match self.next().get_token_type() {
            Token::LeftBracket => {
                let mut sizes = size;
                self.consume();
                let s = self
                    .next()
                    .get_token_value()
                    .parse::<usize>()
                    .expect("failed parse");
                self.must_next(Token::Number, "ast.rs(arra_size): Not exists Number");
                self.must_next(
                    Token::RightBracket,
                    "ast.rs(arra_size): Not exists RightBracket",
                );
                sizes.push(s);
                self.array_size(sizes)
            }
            _ => size,
        }
    }

    // array
    fn variable_array(&mut self, t: Type) -> AstType {
        let token = self.next_consume();
        match token.get_token_type() {
            Token::Variable => {
                // シンボルテーブルへ保存（未登録の場合）.
                let s = Structure::Array(self.array_size(vec![]));
                if self.search_symbol(&self.cur_scope, &token.get_token_value()).is_none() {
                    self.sym_table.register_sym(Symbol::new(
                            self.cur_scope.clone(),
                            token.get_token_value(),
                            t.clone(),
                            s.clone(),
                    ));
                }
                AstType::Variable(t, s, token.get_token_value())
            }
            _ => panic!(
                "ast.rs(variable_array): Not Support Token {:?}",
                self.next()
            ),
        }
    }

    // sizeof演算子
    fn factor_sizeof(&mut self) -> AstType {
        self.must_next(Token::LeftParen, "ast.rs(factor_sizeof): Not exists LeftParen");

        // 次のトークンが型であるか判定
        let token = self.next();
        let ast = match token.get_token_type() {
            Token::Int => {
                self.consume();
                AstType::SizeOf(4)
            }
            Token::Char  => {
                self.consume();
                AstType::SizeOf(1)
            }
            Token::IntPointer | Token::CharPointer => {
                self.consume();
                AstType::SizeOf(8)
            }
            Token::Struct => {
                // シンボルテーブルより、構造体定義を取得し、サイズ算出
                self.consume();
                let name = self.next_consume();
                let sym = self.search_symbol(&self.cur_scope, &name.get_token_value())
                              .expect("cannot search token");
                AstType::SizeOf(sym.size)
            }
            _ => {
                // 型でない場合は、変数や数値リテラル
                let factor = self.factor();

                // サイズを算出し、AST作成
                match factor {
                    AstType::Variable(_, _, _) => {
                        // シンボルテーブルから変数をサーチし、サイズ算出
                        let sym = self.search_symbol(&self.cur_scope, &token.get_token_value()).expect("cannot search token");
                        AstType::SizeOf(sym.size)

                    }
                    AstType::Factor(_) => AstType::SizeOf(8),
                    _ => panic!("{} {}: not supprt ast: {:?}", file!(), line!(), factor)
                }
            }
        };

        self.must_next(Token::RightParen, "ast.rs(factor_sizeof): Not exists LeftParen");
        ast
    }

    // number
    fn number(&self, token: &TokenInfo) -> AstType {
        let n = token.get_token_value().parse::<i64>();
        AstType::Factor(n.expect("ast.rs(number): cannot convert i64"))
    }

    // トークン読み取り.
    fn next(&mut self) -> &'a TokenInfo {
        let n = self.tokens.get(self.current_pos);
        n.expect("ast.rs(next): cannot read next value")
    }

    // 読み取り位置更新.
    fn next_consume(&mut self) -> &'a TokenInfo {
        let token = self.tokens.get(self.current_pos);
        self.current_pos += 1;
        token.expect("ast.rs(next_consume): cannot read next value")
    }

    // 読み取り位置更新.
    fn consume(&mut self) {
        self.current_pos += 1;
    }

    // 読み取り位置巻き戻し.
    fn back(&mut self, i: usize) {
        self.current_pos -= i;
    }

    // 指定されたトークンでない場合、panicメッセージ表示.
    fn must_next(&mut self, t: Token, m: &str) {
        let token = self.next_consume();
        if token.get_token_type() != t {
            panic!("{} {}: {} {:?}", file!(), line!(), m, token)
        }
    }

    // シンボルサーチ
    //
    // ローカルで発見できない場合、グローバルで検索
    fn search_symbol(&self, scope: &Scope, var: &str) -> Option<Symbol> {
        match scope {
            Scope::Global => self.sym_table.search(scope, var),
            _ => {
                let sym = self.sym_table.search(scope, var);
                match sym {
                    Some(_) => sym,
                    _ => self.search_symbol(&Scope::Global, var)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_token(t: Token, s: String) -> TokenInfo {
        TokenInfo::new(t, s, ("".to_string(), 0, 0))
    }

    #[test]
    fn test_add_operator() {
        // 単純な加算テスト.
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::Plus, '+'.to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Plus(
                        Box::new(AstType::Factor(1)),
                        Box::new(AstType::Factor(2))
                    ),])),
                )
            )
        }
        // 複数の加算テスト.
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, '('.to_string()),
                create_token(Token::RightParen, ')'.to_string()),
                create_token(Token::LeftBrace, '{'.to_string()),
                create_token(Token::Number, '1'.to_string()),
                create_token(Token::Plus, '+'.to_string()),
                create_token(Token::Number, '2'.to_string()),
                create_token(Token::Plus, '+'.to_string()),
                create_token(Token::Number, '3'.to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, '}'.to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Plus(
                        Box::new(AstType::Plus(
                            Box::new(AstType::Factor(1)),
                            Box::new(AstType::Factor(2)),
                        )),
                        Box::new(AstType::Factor(3))
                    ),])),
                )
            )
        }
        // 複数の加算テスト.
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, '1'.to_string()),
                create_token(Token::Plus, '+'.to_string()),
                create_token(Token::Number, '2'.to_string()),
                create_token(Token::Plus, '+'.to_string()),
                create_token(Token::Number, '3'.to_string()),
                create_token(Token::Plus, '+'.to_string()),
                create_token(Token::Number, '4'.to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Plus(
                        Box::new(AstType::Plus(
                            Box::new(AstType::Plus(
                                Box::new(AstType::Factor(1)),
                                Box::new(AstType::Factor(2)),
                            )),
                            Box::new(AstType::Factor(3)),
                        )),
                        Box::new(AstType::Factor(4))
                    ),])),
                )
            )
        }
    }

    #[test]
    fn test_sub_operator() {
        // 単純な減算テスト.
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::Minus, '-'.to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Minus(
                        Box::new(AstType::Factor(1)),
                        Box::new(AstType::Factor(2))
                    ),])),
                )
            )
        }
        // 複数の減算テスト.
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "100".to_string()),
                create_token(Token::Minus, '-'.to_string()),
                create_token(Token::Number, '2'.to_string()),
                create_token(Token::Minus, '-'.to_string()),
                create_token(Token::Number, '3'.to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Minus(
                        Box::new(AstType::Minus(
                            Box::new(AstType::Factor(100)),
                            Box::new(AstType::Factor(2)),
                        )),
                        Box::new(AstType::Factor(3))
                    ),])),
                )
            )
        }
        // 複数の減算テスト.
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, '1'.to_string()),
                create_token(Token::Minus, '-'.to_string()),
                create_token(Token::Number, '2'.to_string()),
                create_token(Token::Minus, '-'.to_string()),
                create_token(Token::Number, '3'.to_string()),
                create_token(Token::Minus, '-'.to_string()),
                create_token(Token::Number, '4'.to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "{".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Minus(
                        Box::new(AstType::Minus(
                            Box::new(AstType::Minus(
                                Box::new(AstType::Factor(1)),
                                Box::new(AstType::Factor(2)),
                            )),
                            Box::new(AstType::Factor(3)),
                        )),
                        Box::new(AstType::Factor(4))
                    ),])),
                )
            )
        }
    }

    #[test]
    fn test_mul_operator() {
        // 単純な乗算テスト.
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::Multi, '*'.to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Multiple(
                        Box::new(AstType::Factor(1)),
                        Box::new(AstType::Factor(2))
                    ),])),
                )
            )
        }
        // 複数の減算テスト.
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, '1'.to_string()),
                create_token(Token::Multi, '*'.to_string()),
                create_token(Token::Number, '2'.to_string()),
                create_token(Token::Multi, '*'.to_string()),
                create_token(Token::Number, '3'.to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Multiple(
                        Box::new(AstType::Multiple(
                            Box::new(AstType::Factor(1)),
                            Box::new(AstType::Factor(2)),
                        )),
                        Box::new(AstType::Factor(3))
                    ),])),
                )
            )
        }
        // 複数の減算テスト.
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, '1'.to_string()),
                create_token(Token::Multi, '*'.to_string()),
                create_token(Token::Number, '2'.to_string()),
                create_token(Token::Multi, '*'.to_string()),
                create_token(Token::Number, '3'.to_string()),
                create_token(Token::Multi, '*'.to_string()),
                create_token(Token::Number, '4'.to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Multiple(
                        Box::new(AstType::Multiple(
                            Box::new(AstType::Multiple(
                                Box::new(AstType::Factor(1)),
                                Box::new(AstType::Factor(2)),
                            )),
                            Box::new(AstType::Factor(3)),
                        )),
                        Box::new(AstType::Factor(4))
                    ),])),
                )
            )
        }
    }

    #[test]
    fn test_div_operator() {
        // 単純な乗算テスト.
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::Division, '/'.to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Division(
                        Box::new(AstType::Factor(1)),
                        Box::new(AstType::Factor(2))
                    ),])),
                )
            )
        }
        // 複数の減算テスト.
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, '1'.to_string()),
                create_token(Token::Division, '/'.to_string()),
                create_token(Token::Number, '2'.to_string()),
                create_token(Token::Division, '/'.to_string()),
                create_token(Token::Number, '3'.to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Division(
                        Box::new(AstType::Division(
                            Box::new(AstType::Factor(1)),
                            Box::new(AstType::Factor(2)),
                        )),
                        Box::new(AstType::Factor(3))
                    ),])),
                )
            )
        }
        // 複数の減算テスト.
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, '1'.to_string()),
                create_token(Token::Division, '/'.to_string()),
                create_token(Token::Number, '2'.to_string()),
                create_token(Token::Division, '/'.to_string()),
                create_token(Token::Number, '3'.to_string()),
                create_token(Token::Division, '/'.to_string()),
                create_token(Token::Number, '4'.to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Division(
                        Box::new(AstType::Division(
                            Box::new(AstType::Division(
                                Box::new(AstType::Factor(1)),
                                Box::new(AstType::Factor(2)),
                            )),
                            Box::new(AstType::Factor(3)),
                        )),
                        Box::new(AstType::Factor(4))
                    ),])),
                )
            )
        }
    }

    #[test]
    fn test_mix_operator() {
        // 複数演算子のテスト.
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, '1'.to_string()),
                create_token(Token::Multi, '*'.to_string()),
                create_token(Token::Number, '2'.to_string()),
                create_token(Token::Plus, '+'.to_string()),
                create_token(Token::Number, '3'.to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Plus(
                        Box::new(AstType::Multiple(
                            Box::new(AstType::Factor(1)),
                            Box::new(AstType::Factor(2)),
                        )),
                        Box::new(AstType::Factor(3))
                    ),])),
                )
            )
        }
        // 複数演算子のテスト.
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, '1'.to_string()),
                create_token(Token::Plus, '+'.to_string()),
                create_token(Token::Number, '2'.to_string()),
                create_token(Token::Multi, '*'.to_string()),
                create_token(Token::Number, '3'.to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "{".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Plus(
                        Box::new(AstType::Factor(1)),
                        Box::new(AstType::Multiple(
                            Box::new(AstType::Factor(2)),
                            Box::new(AstType::Factor(3)),
                        ))
                    ),])),
                )
            )
        }
        // 複数演算子のテスト.
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, '1'.to_string()),
                create_token(Token::Division, '/'.to_string()),
                create_token(Token::Number, '2'.to_string()),
                create_token(Token::Plus, '+'.to_string()),
                create_token(Token::Number, '3'.to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Plus(
                        Box::new(AstType::Division(
                            Box::new(AstType::Factor(1)),
                            Box::new(AstType::Factor(2)),
                        )),
                        Box::new(AstType::Factor(3))
                    ),])),
                )
            )
        }
        // 複数演算子のテスト.
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, '1'.to_string()),
                create_token(Token::Plus, '+'.to_string()),
                create_token(Token::Number, '2'.to_string()),
                create_token(Token::Division, '/'.to_string()),
                create_token(Token::Number, '3'.to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Plus(
                        Box::new(AstType::Factor(1)),
                        Box::new(AstType::Division(
                            Box::new(AstType::Factor(2)),
                            Box::new(AstType::Factor(3)),
                        ))
                    ),])),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::LessThan, "<".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::Equal, "==".to_string()),
                create_token(Token::Number, "4".to_string()),
                create_token(Token::GreaterThanEqual, ">=".to_string()),
                create_token(Token::Number, "5".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::GreaterThanEqual(
                        Box::new(AstType::Equal(
                            Box::new(AstType::LessThan(
                                Box::new(AstType::Factor(2)),
                                Box::new(AstType::Factor(3)),
                            )),
                            Box::new(AstType::Factor(4)),
                        )),
                        Box::new(AstType::Factor(5))
                    ),])),
                )
            )
        }
    }

    #[test]
    fn test_bracket() {
        // カッコのテスト.
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::Plus, '+'.to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Plus(
                        Box::new(AstType::Factor(1)),
                        Box::new(AstType::Factor(2))
                    ),])),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::Plus, '+'.to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::Plus, '+'.to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Plus(
                        Box::new(AstType::Factor(1)),
                        Box::new(AstType::Plus(
                            Box::new(AstType::Factor(2)),
                            Box::new(AstType::Factor(3)),
                        ))
                    ),])),
                )
            )
        }
    }

    #[test]
    fn test_equal_operator() {
        // 等価演算子テスト.
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::Plus, '+'.to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::Equal, "==".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::Plus, '+'.to_string()),
                create_token(Token::Number, "4".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Equal(
                        Box::new(AstType::Plus(
                            Box::new(AstType::Factor(1)),
                            Box::new(AstType::Factor(2)),
                        )),
                        Box::new(AstType::Plus(
                            Box::new(AstType::Factor(3)),
                            Box::new(AstType::Factor(4)),
                        ))
                    ),])),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::Multi, '*'.to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::Equal, "==".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::Multi, '*'.to_string()),
                create_token(Token::Number, "4".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Equal(
                        Box::new(AstType::Multiple(
                            Box::new(AstType::Factor(1)),
                            Box::new(AstType::Factor(2)),
                        )),
                        Box::new(AstType::Multiple(
                            Box::new(AstType::Factor(3)),
                            Box::new(AstType::Factor(4)),
                        ))
                    ),])),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::Multi, '*'.to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::Plus, '+'.to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::Equal, "==".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::Minus, '-'.to_string()),
                create_token(Token::Number, "4".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Equal(
                        Box::new(AstType::Plus(
                            Box::new(AstType::Multiple(
                                Box::new(AstType::Factor(1)),
                                Box::new(AstType::Factor(2)),
                            )),
                            Box::new(AstType::Factor(1)),
                        )),
                        Box::new(AstType::Minus(
                            Box::new(AstType::Factor(3)),
                            Box::new(AstType::Factor(4)),
                        ))
                    ),])),
                )
            )
        }
    }

    #[test]
    fn test_not_equal_operator() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::Multi, '*'.to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::Plus, '+'.to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::NotEqual, "!=".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::Minus, '-'.to_string()),
                create_token(Token::Number, "4".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::NotEqual(
                        Box::new(AstType::Plus(
                            Box::new(AstType::Multiple(
                                Box::new(AstType::Factor(1)),
                                Box::new(AstType::Factor(2)),
                            )),
                            Box::new(AstType::Factor(1)),
                        )),
                        Box::new(AstType::Minus(
                            Box::new(AstType::Factor(3)),
                            Box::new(AstType::Factor(4)),
                        ))
                    ),])),
                )
            )
        }
    }

    #[test]
    fn test_less_than_operator() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::Multi, '*'.to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::Plus, '+'.to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::LessThan, "<".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::Minus, '-'.to_string()),
                create_token(Token::Number, "4".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::LessThan(
                        Box::new(AstType::Plus(
                            Box::new(AstType::Multiple(
                                Box::new(AstType::Factor(1)),
                                Box::new(AstType::Factor(2)),
                            )),
                            Box::new(AstType::Factor(1)),
                        )),
                        Box::new(AstType::Minus(
                            Box::new(AstType::Factor(3)),
                            Box::new(AstType::Factor(4)),
                        ))
                    ),])),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::Multi, '*'.to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::Plus, '+'.to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::LessThanEqual, "<=".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::Minus, '-'.to_string()),
                create_token(Token::Number, "4".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::LessThanEqual(
                        Box::new(AstType::Plus(
                            Box::new(AstType::Multiple(
                                Box::new(AstType::Factor(1)),
                                Box::new(AstType::Factor(2)),
                            )),
                            Box::new(AstType::Factor(1)),
                        )),
                        Box::new(AstType::Minus(
                            Box::new(AstType::Factor(3)),
                            Box::new(AstType::Factor(4)),
                        ))
                    ),])),
                )
            )
        }
    }

    #[test]
    fn test_greater_than_operator() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::Multi, '*'.to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::Plus, '+'.to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::GreaterThan, ">".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::Minus, '-'.to_string()),
                create_token(Token::Number, "4".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::GreaterThan(
                        Box::new(AstType::Plus(
                            Box::new(AstType::Multiple(
                                Box::new(AstType::Factor(1)),
                                Box::new(AstType::Factor(2)),
                            )),
                            Box::new(AstType::Factor(1)),
                        )),
                        Box::new(AstType::Minus(
                            Box::new(AstType::Factor(3)),
                            Box::new(AstType::Factor(4)),
                        ))
                    ),])),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::Multi, '*'.to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::Plus, '+'.to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::GreaterThanEqual, ">=".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::Minus, '-'.to_string()),
                create_token(Token::Number, "4".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::GreaterThanEqual(
                        Box::new(AstType::Plus(
                            Box::new(AstType::Multiple(
                                Box::new(AstType::Factor(1)),
                                Box::new(AstType::Factor(2)),
                            )),
                            Box::new(AstType::Factor(1)),
                        )),
                        Box::new(AstType::Minus(
                            Box::new(AstType::Factor(3)),
                            Box::new(AstType::Factor(4)),
                        ))
                    ),])),
                )
            )
        }
    }

    #[test]
    fn test_logical_operator() {
        // &&演算子のテスト.
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::LogicalAnd, "&&".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::LogicalAnd(
                        Box::new(AstType::Factor(2)),
                        Box::new(AstType::Factor(3))
                    ),])),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::Plus, "+".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::LogicalAnd, "&&".to_string()),
                create_token(Token::Number, "4".to_string()),
                create_token(Token::Plus, "+".to_string()),
                create_token(Token::Number, "5".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::LogicalAnd(
                        Box::new(AstType::Plus(
                            Box::new(AstType::Factor(2)),
                            Box::new(AstType::Factor(3)),
                        )),
                        Box::new(AstType::Plus(
                            Box::new(AstType::Factor(4)),
                            Box::new(AstType::Factor(5)),
                        ))
                    ),])),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::Plus, "+".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::Equal, "==".to_string()),
                create_token(Token::Number, "4".to_string()),
                create_token(Token::Plus, "+".to_string()),
                create_token(Token::Number, "5".to_string()),
                create_token(Token::LogicalAnd, "&&".to_string()),
                create_token(Token::Number, "6".to_string()),
                create_token(Token::Plus, "+".to_string()),
                create_token(Token::Number, "7".to_string()),
                create_token(Token::NotEqual, "!=".to_string()),
                create_token(Token::Number, "8".to_string()),
                create_token(Token::Plus, "+".to_string()),
                create_token(Token::Number, "9".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::LogicalAnd(
                        Box::new(AstType::Equal(
                            Box::new(AstType::Plus(
                                Box::new(AstType::Factor(2)),
                                Box::new(AstType::Factor(3)),
                            )),
                            Box::new(AstType::Plus(
                                Box::new(AstType::Factor(4)),
                                Box::new(AstType::Factor(5)),
                            )),
                        )),
                        Box::new(AstType::NotEqual(
                            Box::new(AstType::Plus(
                                Box::new(AstType::Factor(6)),
                                Box::new(AstType::Factor(7)),
                            )),
                            Box::new(AstType::Plus(
                                Box::new(AstType::Factor(8)),
                                Box::new(AstType::Factor(9)),
                            )),
                        ))
                    ),])),
                )
            )
        }
        // ||演算子のテスト.
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::LogicalOr, "||".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::LogicalOr(
                        Box::new(AstType::Factor(2)),
                        Box::new(AstType::Factor(3))
                    ),])),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::Plus, "+".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::LogicalOr, "||".to_string()),
                create_token(Token::Number, "4".to_string()),
                create_token(Token::Plus, "+".to_string()),
                create_token(Token::Number, "5".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::LogicalOr(
                        Box::new(AstType::Plus(
                            Box::new(AstType::Factor(2)),
                            Box::new(AstType::Factor(3)),
                        )),
                        Box::new(AstType::Plus(
                            Box::new(AstType::Factor(4)),
                            Box::new(AstType::Factor(5)),
                        ))
                    ),])),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::Plus, "+".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::Equal, "==".to_string()),
                create_token(Token::Number, "4".to_string()),
                create_token(Token::Plus, "+".to_string()),
                create_token(Token::Number, "5".to_string()),
                create_token(Token::LogicalOr, "||".to_string()),
                create_token(Token::Number, "6".to_string()),
                create_token(Token::Plus, "+".to_string()),
                create_token(Token::Number, "7".to_string()),
                create_token(Token::NotEqual, "!=".to_string()),
                create_token(Token::Number, "8".to_string()),
                create_token(Token::Plus, "+".to_string()),
                create_token(Token::Number, "9".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::LogicalOr(
                        Box::new(AstType::Equal(
                            Box::new(AstType::Plus(
                                Box::new(AstType::Factor(2)),
                                Box::new(AstType::Factor(3)),
                            )),
                            Box::new(AstType::Plus(
                                Box::new(AstType::Factor(4)),
                                Box::new(AstType::Factor(5)),
                            )),
                        )),
                        Box::new(AstType::NotEqual(
                            Box::new(AstType::Plus(
                                Box::new(AstType::Factor(6)),
                                Box::new(AstType::Factor(7)),
                            )),
                            Box::new(AstType::Plus(
                                Box::new(AstType::Factor(8)),
                                Box::new(AstType::Factor(9)),
                            )),
                        ))
                    ),])),
                )
            )
        }
    }

    #[test]
    fn test_mix_logical_operator() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::LogicalOr, "||".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::LogicalAnd, "&&".to_string()),
                create_token(Token::Number, "4".to_string()),
                create_token(Token::LogicalOr, "||".to_string()),
                create_token(Token::Number, "5".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::LogicalOr(
                        Box::new(AstType::LogicalAnd(
                            Box::new(AstType::LogicalOr(
                                Box::new(AstType::Factor(2)),
                                Box::new(AstType::Factor(3)),
                            )),
                            Box::new(AstType::Factor(4)),
                        )),
                        Box::new(AstType::Factor(5))
                    ),])),
                )
            )
        }
    }

    #[test]
    fn test_condition_expression() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::Equal, "==".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::Question, "?".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::Colon, ":".to_string()),
                create_token(Token::Number, "5".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Condition(
                        Box::new(AstType::Equal(
                            Box::new(AstType::Factor(2)),
                            Box::new(AstType::Factor(3)),
                        )),
                        Box::new(AstType::Factor(1)),
                        Box::new(AstType::Factor(5))
                    ),])),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::Equal, "==".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::Question, "?".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::Number, "10".to_string()),
                create_token(Token::Equal, "==".to_string()),
                create_token(Token::Number, "11".to_string()),
                create_token(Token::Question, "?".to_string()),
                create_token(Token::Number, "12".to_string()),
                create_token(Token::Colon, ":".to_string()),
                create_token(Token::Number, "13".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::Colon, ":".to_string()),
                create_token(Token::Number, "5".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Condition(
                        Box::new(AstType::Equal(
                            Box::new(AstType::Factor(2)),
                            Box::new(AstType::Factor(3)),
                        )),
                        Box::new(AstType::Condition(
                            Box::new(AstType::Equal(
                                Box::new(AstType::Factor(10)),
                                Box::new(AstType::Factor(11)),
                            )),
                            Box::new(AstType::Factor(12)),
                            Box::new(AstType::Factor(13)),
                        )),
                        Box::new(AstType::Factor(5))
                    ),])),
                )
            )
        }
    }

    #[test]
    fn test_unary_operator() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Plus, "+".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::UnPlus(Box::new(
                        AstType::Factor(2)
                    ))],)),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Plus, "+".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::Minus, "-".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Minus(
                        Box::new(AstType::UnPlus(Box::new(AstType::Factor(2)))),
                        Box::new(AstType::Factor(1))
                    ),])),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::Plus, "+".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::Minus, "-".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Minus(
                        Box::new(AstType::UnPlus(Box::new(AstType::Factor(2)))),
                        Box::new(AstType::Factor(1))
                    ),])),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Plus, "+".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::Multi, "*".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Multiple(
                        Box::new(AstType::UnPlus(Box::new(AstType::Factor(2)))),
                        Box::new(AstType::Factor(1))
                    ),])),
                )
            )
        }
        // 否定演算子のテスト.
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Not, "!".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Not(Box::new(
                        AstType::Factor(2)
                    ))])),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Not, "!".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::Equal, "==".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Not(Box::new(
                        AstType::Equal(Box::new(AstType::Factor(2)), Box::new(AstType::Factor(3)),)
                    )),])),
                )
            )
        }
        // ビット反転演算子.
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::BitReverse, "~".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::BitReverse(Box::new(
                        AstType::Factor(2)
                    ))],)),
                )
            )
        }
    }

    #[test]
    fn test_shift_operator() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::LeftShift, "<<".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::LeftShift(
                        Box::new(AstType::Factor(2)),
                        Box::new(AstType::Factor(1))
                    ),])),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::RightShift, ">>".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::RightShift(
                        Box::new(AstType::Factor(2)),
                        Box::new(AstType::Factor(1))
                    ),])),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::Plus, "+".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::RightShift, ">>".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::RightShift(
                        Box::new(AstType::Plus(
                            Box::new(AstType::Factor(2)),
                            Box::new(AstType::Factor(3)),
                        )),
                        Box::new(AstType::Factor(1))
                    ),])),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::LessThan, "<".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::RightShift, ">>".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::LessThan(
                        Box::new(AstType::Factor(2)),
                        Box::new(AstType::RightShift(
                            Box::new(AstType::Factor(3)),
                            Box::new(AstType::Factor(1)),
                        ))
                    ),])),
                )
            )
        }
    }

    // ビット演算子テスト.
    #[test]
    fn test_bit_operator() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::And, "&".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::BitAnd(
                        Box::new(AstType::Factor(2)),
                        Box::new(AstType::Factor(3))
                    ),])),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::BitOr, "&".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::BitOr(
                        Box::new(AstType::Factor(2)),
                        Box::new(AstType::Factor(3))
                    ),])),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::BitXor, "^".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::BitXor(
                        Box::new(AstType::Factor(2)),
                        Box::new(AstType::Factor(3))
                    ),])),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::And, "&".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::BitOr, "|".to_string()),
                create_token(Token::Number, "4".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::BitOr(
                        Box::new(AstType::BitAnd(
                            Box::new(AstType::Factor(2)),
                            Box::new(AstType::Factor(3)),
                        )),
                        Box::new(AstType::Factor(4))
                    ),])),
                )
            )
        }
    }

    #[test]
    fn test_assign_operator() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Assign(
                        Box::new(AstType::Variable(
                            Type::Int,
                            Structure::Identifier,
                            "a".to_string()
                        )),
                        Box::new(AstType::Factor(3))
                    ),])),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::Plus, "+".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Assign(
                        Box::new(AstType::Variable(
                            Type::Int,
                            Structure::Identifier,
                            "a".to_string()
                        )),
                        Box::new(AstType::Plus(
                            Box::new(AstType::Factor(3)),
                            Box::new(AstType::Factor(1)),
                        ))
                    ),])),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::LogicalAnd, "&&".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Identifier, "a".to_string()),
                        AstType::Assign(
                            Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Identifier,
                                "a".to_string()
                            )),
                            Box::new(AstType::LogicalAnd(
                                Box::new(AstType::Factor(3)),
                                Box::new(AstType::Factor(1)),
                            ))
                        ),
                    ])),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::Multi, "*".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Assign(
                        Box::new(AstType::Variable(
                            Type::Int,
                            Structure::Identifier,
                            "a".to_string()
                        )),
                        Box::new(AstType::Multiple(
                            Box::new(AstType::Factor(3)),
                            Box::new(AstType::Factor(1)),
                        ))
                    ),])),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::BitOr, "|".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Assign(
                        Box::new(AstType::Variable(
                            Type::Int,
                            Structure::Identifier,
                            "a".to_string()
                        )),
                        Box::new(AstType::BitOr(
                            Box::new(AstType::Factor(3)),
                            Box::new(AstType::Factor(1)),
                        ))
                    ),])),
                )
            )
        }
    }

    #[test]
    fn test_call_func() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "a".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![])),
                )
            );
            assert_eq!(
                result.get_tree()[1],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::FuncCall(
                        Box::new(AstType::Variable(
                            Type::Int,
                            Structure::Identifier,
                            "a".to_string()
                        )),
                        Box::new(AstType::Argment(vec![]))
                    ),])),
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "x".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "a".to_string(),
                    Box::new(AstType::Argment(vec![AstType::Variable(
                        Type::Int,
                        Structure::Identifier,
                        "x".to_string()
                    ),])),
                    Box::new(AstType::Statement(vec![]))
                )
            );
            assert_eq!(
                result.get_tree()[1],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Identifier, "b".to_string()),
                        AstType::FuncCall(
                            Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Identifier,
                                "a".to_string()
                            )),
                            Box::new(AstType::Argment(vec![AstType::Variable(
                                Type::Int,
                                Structure::Identifier,
                                'b'.to_string()
                            )]),)
                        ),
                    ])),
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "test".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "x".to_string()),
                create_token(Token::Comma, ",".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "y".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "c".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Variable, "test".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::Comma, ",".to_string()),
                create_token(Token::Variable, "c".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "test".to_string(),
                    Box::new(AstType::Argment(vec![
                        AstType::Variable(Type::Int, Structure::Identifier, "x".to_string()),
                        AstType::Variable(Type::Int, Structure::Identifier, "y".to_string()),
                    ])),
                    Box::new(AstType::Statement(vec![]))
                )
            );
            assert_eq!(
                result.get_tree()[1],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Identifier, 'b'.to_string()),
                        AstType::Variable(Type::Int, Structure::Identifier, 'c'.to_string()),
                        AstType::FuncCall(
                            Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Identifier,
                                "test".to_string()
                            )),
                            Box::new(AstType::Argment(vec![
                                AstType::Variable(
                                    Type::Int,
                                    Structure::Identifier,
                                    'b'.to_string()
                                ),
                                AstType::Variable(
                                    Type::Int,
                                    Structure::Identifier,
                                    'c'.to_string()
                                ),
                            ]))
                        ),
                    ])),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::IntPointer, "int*".to_string()),
                create_token(Token::Variable, "x".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "y".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::And, "&".to_string()),
                create_token(Token::Variable, "y".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "a".to_string(),
                    Box::new(AstType::Argment(vec![AstType::Variable(
                        Type::Int,
                        Structure::Pointer,
                        "x".to_string()
                    ),])),
                    Box::new(AstType::Statement(vec![])),
                )
            );
            assert_eq!(
                result.get_tree()[1],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Identifier, "y".to_string()),
                        AstType::FuncCall(
                            Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Identifier,
                                "a".to_string()
                            )),
                            Box::new(AstType::Argment(vec![AstType::Address(Box::new(
                                AstType::Variable(
                                    Type::Int,
                                    Structure::Identifier,
                                    'y'.to_string()
                                )
                            ))]))
                        ),
                    ])),
                )
            );
        }
    }

    #[test]
    fn test_compound() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Plus, "+".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "{".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Assign(
                            Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Identifier,
                                "a".to_string()
                            )),
                            Box::new(AstType::Factor(3))
                        ),
                        AstType::Assign(
                            Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Identifier,
                                "a".to_string()
                            )),
                            Box::new(AstType::Plus(
                                Box::new(AstType::Variable(
                                    Type::Int,
                                    Structure::Identifier,
                                    "a".to_string()
                                )),
                                Box::new(AstType::Factor(3)),
                            ))
                        ),
                    ])),
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Multi, "*".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Plus, "+".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Assign(
                            Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Identifier,
                                "a".to_string()
                            )),
                            Box::new(AstType::Factor(3))
                        ),
                        AstType::Plus(
                            Box::new(AstType::Multiple(
                                Box::new(AstType::Variable(
                                    Type::Int,
                                    Structure::Identifier,
                                    "a".to_string()
                                )),
                                Box::new(AstType::Variable(
                                    Type::Int,
                                    Structure::Identifier,
                                    "a".to_string()
                                )),
                            )),
                            Box::new(AstType::Factor(1))
                        ),
                    ])),
                )
            )
        }
    }

    #[test]
    fn test_some_func_def() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "test".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Assign(
                        Box::new(AstType::Variable(
                            Type::Int,
                            Structure::Identifier,
                            "a".to_string()
                        )),
                        Box::new(AstType::Factor(3))
                    ),])),
                )
            );
            assert_eq!(
                result.get_tree()[1],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "test".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Assign(
                        Box::new(AstType::Variable(
                            Type::Int,
                            Structure::Identifier,
                            "b".to_string()
                        )),
                        Box::new(AstType::Factor(1))
                    ),])),
                )
            );
        }
    }

    #[test]
    fn test_func_def_with_args() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Comma, ",".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "c".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![
                        AstType::Variable(Type::Int, Structure::Identifier, "a".to_string()),
                        AstType::Variable(Type::Int, Structure::Identifier, "b".to_string()),
                    ])),
                    Box::new(AstType::Statement(vec![AstType::Assign(
                        Box::new(AstType::Variable(
                            Type::Int,
                            Structure::Identifier,
                            "c".to_string()
                        )),
                        Box::new(AstType::Factor(3))
                    ),])),
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::IntPointer, "int*".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Comma, ",".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "c".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![
                        AstType::Variable(Type::Int, Structure::Pointer, "a".to_string()),
                        AstType::Variable(Type::Int, Structure::Identifier, "b".to_string()),
                    ])),
                    Box::new(AstType::Statement(vec![AstType::Assign(
                        Box::new(AstType::Variable(
                            Type::Int,
                            Structure::Identifier,
                            "c".to_string()
                        )),
                        Box::new(AstType::Factor(3))
                    ),])),
                )
            );
        }
    }

    #[test]
    fn test_statement_if() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::If, "if".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Equal, "==".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "10".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Identifier, "a".to_string()),
                        AstType::If(
                            Box::new(AstType::Equal(
                                Box::new(AstType::Variable(
                                    Type::Int,
                                    Structure::Identifier,
                                    "a".to_string()
                                )),
                                Box::new(AstType::Factor(3))
                            )),
                            Box::new(AstType::Statement(vec![
                                AstType::Factor(1),
                                AstType::Assign(
                                    Box::new(AstType::Variable(
                                        Type::Int,
                                        Structure::Identifier,
                                        "b".to_string()
                                    )),
                                    Box::new(AstType::Factor(10))
                                )
                            ],)),
                            Box::new(None)
                        )
                    ]))
                )
            );
        }
    }

    #[test]
    fn test_statement_else() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "e".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::If, "if".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Equal, "==".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "10".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::Else, "else".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Variable, "e".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "9".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Identifier, "a".to_string()),
                        AstType::Variable(Type::Int, Structure::Identifier, "b".to_string()),
                        AstType::Variable(Type::Int, Structure::Identifier, "e".to_string()),
                        AstType::If(
                            Box::new(AstType::Equal(
                                Box::new(AstType::Variable(
                                    Type::Int,
                                    Structure::Identifier,
                                    "a".to_string()
                                )),
                                Box::new(AstType::Factor(3))
                            )),
                            Box::new(AstType::Statement(vec![
                                AstType::Factor(1),
                                AstType::Assign(
                                    Box::new(AstType::Variable(
                                        Type::Int,
                                        Structure::Identifier,
                                        "b".to_string()
                                    )),
                                    Box::new(AstType::Factor(10))
                                )
                            ],)),
                            Box::new(Some(AstType::Statement(vec![AstType::Assign(
                                Box::new(AstType::Variable(
                                    Type::Int,
                                    Structure::Identifier,
                                    "e".to_string()
                                )),
                                Box::new(AstType::Factor(9))
                            )],))),
                        ),
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::If, "if".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Equal, "==".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Else, "else".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "e".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "9".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Identifier, "a".to_string()),
                        AstType::If(
                            Box::new(AstType::Equal(
                                Box::new(AstType::Variable(
                                    Type::Int,
                                    Structure::Identifier,
                                    "a".to_string()
                                )),
                                Box::new(AstType::Factor(3))
                            )),
                            Box::new(AstType::Statement(vec![AstType::Factor(1)])),
                            Box::new(Some(AstType::Statement(vec![AstType::Assign(
                                Box::new(AstType::Variable(
                                    Type::Int,
                                    Structure::Identifier,
                                    "e".to_string()
                                )),
                                Box::new(AstType::Factor(9))
                            )])))
                        ),
                    ]))
                )
            );
        }
    }

    #[test]
    fn test_statement_while() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::While, "while".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Equal, "==".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "10".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Identifier, "a".to_string()),
                        AstType::While(
                            Box::new(AstType::Equal(
                                Box::new(AstType::Variable(
                                    Type::Int,
                                    Structure::Identifier,
                                    "a".to_string()
                                )),
                                Box::new(AstType::Factor(3))
                            )),
                            Box::new(AstType::Statement(vec![
                                AstType::Factor(1),
                                AstType::Assign(
                                    Box::new(AstType::Variable(
                                        Type::Int,
                                        Structure::Identifier,
                                        "b".to_string()
                                    )),
                                    Box::new(AstType::Factor(10))
                                )
                            ]))
                        )
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::While, "while".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Equal, "==".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "10".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Identifier, "a".to_string()),
                        AstType::While(
                            Box::new(AstType::Equal(
                                Box::new(AstType::Variable(
                                    Type::Int,
                                    Structure::Identifier,
                                    "a".to_string()
                                )),
                                Box::new(AstType::Factor(3))
                            )),
                            Box::new(AstType::Statement(vec![
                                AstType::Factor(1),
                                AstType::Assign(
                                    Box::new(AstType::Variable(
                                        Type::Int,
                                        Structure::Identifier,
                                        "b".to_string()
                                    )),
                                    Box::new(AstType::Factor(10))
                                )
                            ],))
                        ),
                        AstType::Variable(Type::Int, Structure::Identifier, "b".to_string())
                    ]))
                )
            );
        }
    }

    #[test]
    fn test_statement_for() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::For, "for".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "10".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::For(
                        Box::new(None),
                        Box::new(None),
                        Box::new(None),
                        Box::new(AstType::Statement(vec![
                            AstType::Factor(1),
                            AstType::Assign(
                                Box::new(AstType::Variable(
                                    Type::Int,
                                    Structure::Identifier,
                                    "b".to_string()
                                )),
                                Box::new(AstType::Factor(10))
                            )
                        ],))
                    )]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::For, "for".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "i".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "0".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Variable, "i".to_string()),
                create_token(Token::LessThan, "<".to_string()),
                create_token(Token::Number, "10".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Variable, "i".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Variable, "i".to_string()),
                create_token(Token::Plus, "+".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "10".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::For(
                        Box::new(Some(AstType::Assign(
                            Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Identifier,
                                "i".to_string()
                            )),
                            Box::new(AstType::Factor(0))
                        ),)),
                        Box::new(Some(AstType::LessThan(
                            Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Identifier,
                                "i".to_string()
                            )),
                            Box::new(AstType::Factor(10))
                        ),)),
                        Box::new(Some(AstType::Assign(
                            Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Identifier,
                                "i".to_string()
                            )),
                            Box::new(AstType::Plus(
                                Box::new(AstType::Variable(
                                    Type::Int,
                                    Structure::Identifier,
                                    "i".to_string()
                                )),
                                Box::new(AstType::Factor(1))
                            ))
                        ))),
                        Box::new(AstType::Statement(vec![
                            AstType::Factor(1),
                            AstType::Assign(
                                Box::new(AstType::Variable(
                                    Type::Int,
                                    Structure::Identifier,
                                    "b".to_string()
                                )),
                                Box::new(AstType::Factor(10))
                            )
                        ],))
                    )]))
                )
            );
        }
    }

    #[test]
    fn test_statement_do_while() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Do, "do".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "10".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::While, "while".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Equal, "==".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Do(
                        Box::new(AstType::Statement(vec![
                            AstType::Factor(1),
                            AstType::Variable(Type::Int, Structure::Identifier, "a".to_string()),
                            AstType::Assign(
                                Box::new(AstType::Variable(
                                    Type::Int,
                                    Structure::Identifier,
                                    "b".to_string()
                                )),
                                Box::new(AstType::Factor(10))
                            )
                        ],)),
                        Box::new(AstType::Equal(
                            Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Identifier,
                                "a".to_string()
                            )),
                            Box::new(AstType::Factor(3))
                        )),
                    )]))
                )
            );
        }
    }

    #[test]
    fn test_statement_continue_and_break() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Do, "do".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "10".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Continue, "continue".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::While, "while".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Equal, "==".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Do(
                        Box::new(AstType::Statement(vec![
                            AstType::Factor(1),
                            AstType::Variable(Type::Int, Structure::Identifier, "a".to_string()),
                            AstType::Assign(
                                Box::new(AstType::Variable(
                                    Type::Int,
                                    Structure::Identifier,
                                    "b".to_string()
                                )),
                                Box::new(AstType::Factor(10))
                            ),
                            AstType::Continue(),
                        ],)),
                        Box::new(AstType::Equal(
                            Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Identifier,
                                "a".to_string()
                            )),
                            Box::new(AstType::Factor(3))
                        )),
                    )]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Do, "do".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "10".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Break, "break".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::While, "while".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Equal, "==".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Do(
                        Box::new(AstType::Statement(vec![
                            AstType::Variable(Type::Int, Structure::Identifier, "a".to_string()),
                            AstType::Factor(1),
                            AstType::Assign(
                                Box::new(AstType::Variable(
                                    Type::Int,
                                    Structure::Identifier,
                                    "b".to_string()
                                )),
                                Box::new(AstType::Factor(10))
                            ),
                            AstType::Break(),
                        ],)),
                        Box::new(AstType::Equal(
                            Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Identifier,
                                "a".to_string()
                            )),
                            Box::new(AstType::Factor(3))
                        )),
                    )]))
                )
            );
        }
    }

    #[test]
    fn test_statement_return() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::Equal, "==".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Return(Box::new(
                        AstType::Equal(Box::new(AstType::Factor(1)), Box::new(AstType::Factor(2)))
                    ))]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Identifier, "a".to_string()),
                        AstType::Return(Box::new(AstType::Variable(
                            Type::Int,
                            Structure::Identifier,
                            "a".to_string()
                        )))
                    ]))
                )
            );
        }
    }

    #[test]
    fn test_address_indirect() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::And, "&".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Identifier, "a".to_string()),
                        AstType::Assign(
                            Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Identifier,
                                "a".to_string()
                            )),
                            Box::new(AstType::Address(Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Identifier,
                                "a".to_string()
                            )))),
                        ),
                        AstType::Return(Box::new(AstType::Variable(
                            Type::Int,
                            Structure::Identifier,
                            "a".to_string()
                        )))
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Multi, "*".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Identifier, "a".to_string()),
                        AstType::Assign(
                            Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Identifier,
                                "a".to_string()
                            )),
                            Box::new(AstType::Indirect(Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Identifier,
                                "a".to_string()
                            )))),
                        ),
                        AstType::Return(Box::new(AstType::Variable(
                            Type::Int,
                            Structure::Identifier,
                            "a".to_string()
                        )))
                    ]))
                )
            );
        }
    }

    #[test]
    fn test_type_pointer() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::IntPointer, "int*".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Pointer, "a".to_string()),
                        AstType::Return(Box::new(AstType::Variable(
                            Type::Int,
                            Structure::Pointer,
                            "a".to_string()
                        )))
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::IntPointer, "int*".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Pointer, "a".to_string()),
                        AstType::Return(Box::new(AstType::Variable(
                            Type::Int,
                            Structure::Pointer,
                            "a".to_string()
                        )))
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::IntPointer, "int*".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Multi, "*".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Multi, "*".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Plus, "+".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Pointer, "a".to_string()),
                        AstType::Assign(
                            Box::new(AstType::Indirect(Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Pointer,
                                "a".to_string()
                            )),)),
                            Box::new(AstType::Indirect(Box::new(AstType::Plus(
                                Box::new(AstType::Variable(
                                    Type::Int,
                                    Structure::Pointer,
                                    "a".to_string()
                                )),
                                Box::new(AstType::Factor(1))
                            )),))
                        ),
                        AstType::Return(Box::new(AstType::Variable(
                            Type::Int,
                            Structure::Pointer,
                            "a".to_string()
                        )))
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::IntPointer, "int*".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Multi, "*".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Multi, "*".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Plus, "+".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Pointer, "a".to_string()),
                        AstType::Assign(
                            Box::new(AstType::Indirect(Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Pointer,
                                "a".to_string()
                            )),)),
                            Box::new(AstType::Plus(
                                Box::new(AstType::Indirect(Box::new(AstType::Variable(
                                    Type::Int,
                                    Structure::Pointer,
                                    "a".to_string()
                                )))),
                                Box::new(AstType::Factor(1))
                            )),
                        ),
                        AstType::Return(Box::new(AstType::Variable(
                            Type::Int,
                            Structure::Pointer,
                            "a".to_string()
                        )))
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::IntPointer, "int*".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Multi, "*".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Multi, "*".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Minus, "-".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Pointer, "a".to_string()),
                        AstType::Assign(
                            Box::new(AstType::Indirect(Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Pointer,
                                "a".to_string()
                            )),)),
                            Box::new(AstType::Minus(
                                Box::new(AstType::Indirect(Box::new(AstType::Variable(
                                    Type::Int,
                                    Structure::Pointer,
                                    "a".to_string()
                                )))),
                                Box::new(AstType::Factor(1))
                            )),
                        ),
                        AstType::Return(Box::new(AstType::Variable(
                            Type::Int,
                            Structure::Pointer,
                            "a".to_string()
                        )))
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::CharPointer, "char*".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Char, Structure::Pointer, "a".to_string()),
                        AstType::Return(Box::new(AstType::Variable(
                            Type::Char,
                            Structure::Pointer,
                            "a".to_string()
                        )))
                    ]))
                )
            );
        }
    }

    #[test]
    fn test_assign_indirect() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::IntPointer, "int*".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::And, "&".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Identifier, "a".to_string()),
                        AstType::Assign(
                            Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Pointer,
                                "b".to_string()
                            )),
                            Box::new(AstType::Address(Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Identifier,
                                "a".to_string()
                            )),))
                        ),
                        AstType::Return(Box::new(AstType::Factor(1)),)
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::IntPointer, "int*".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::And, "&".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Multi, "*".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "120".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Identifier, "a".to_string()),
                        AstType::Assign(
                            Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Pointer,
                                "b".to_string()
                            )),
                            Box::new(AstType::Address(Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Identifier,
                                "a".to_string()
                            )),))
                        ),
                        AstType::Assign(
                            Box::new(AstType::Indirect(Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Pointer,
                                "b".to_string()
                            )),)),
                            Box::new(AstType::Factor(120)),
                        ),
                        AstType::Return(Box::new(AstType::Factor(1)),)
                    ]))
                )
            );
        }
    }

    #[test]
    fn test_add_sub_indirect() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::IntPointer, "int*".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Plus, "+".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Pointer, "a".to_string()),
                        AstType::Plus(
                            Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Pointer,
                                "a".to_string()
                            )),
                            Box::new(AstType::Factor(1)),
                        ),
                        AstType::Return(Box::new(AstType::Factor(1)),)
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::IntPointer, "int*".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Minus, "+".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Pointer, "a".to_string()),
                        AstType::Minus(
                            Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Pointer,
                                "a".to_string()
                            )),
                            Box::new(AstType::Factor(1)),
                        ),
                        AstType::Return(Box::new(AstType::Factor(1)),)
                    ]))
                )
            );
        }
    }

    #[test]
    fn test_array() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::LeftBracket, " [".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::RightBracket, "]".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Array(vec![3]), "a".to_string()),
                        AstType::Return(Box::new(AstType::Factor(1)),)
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::LeftBracket, " [".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::RightBracket, "]".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::LeftBracket, " [".to_string()),
                create_token(Token::Number, "0".to_string()),
                create_token(Token::RightBracket, "]".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "10".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Array(vec![3]), "a".to_string()),
                        AstType::Assign(
                            Box::new(AstType::Indirect(Box::new(AstType::Plus(
                                Box::new(AstType::Variable(
                                    Type::Int,
                                    Structure::Array(vec![3]),
                                    "a".to_string()
                                )),
                                Box::new(AstType::Factor(0)),
                            )),)),
                            Box::new(AstType::Factor(10)),
                        ),
                        AstType::Return(Box::new(AstType::Factor(1)),)
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::LeftBracket, " [".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::RightBracket, "]".to_string()),
                create_token(Token::LeftBracket, " [".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::RightBracket, "]".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Array(vec![3, 3]), "a".to_string()),
                        AstType::Return(Box::new(AstType::Factor(1)),)
                    ]))
                )
            );
        }
    }

    #[test]
    fn test_variable() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Comma, ",".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Identifier, "a".to_string()),
                        AstType::Variable(Type::Int, Structure::Identifier, "b".to_string()),
                        AstType::Return(Box::new(AstType::Factor(1)),)
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::Global(vec![AstType::Variable(
                    Type::Int,
                    Structure::Identifier,
                    "a".to_string()
                ),]),
            );
            assert_eq!(
                result.get_tree()[1],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Return(Box::new(
                        AstType::Factor(1)
                    ),)]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "100".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::Global(vec![AstType::Assign(
                    Box::new(AstType::Variable(
                        Type::Int,
                        Structure::Identifier,
                        "a".to_string()
                    )),
                    Box::new(AstType::Factor(100)),
                )])
            );
            assert_eq!(
                result.get_tree()[1],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Return(Box::new(
                        AstType::Factor(1)
                    ),)]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::LeftBracket, " [".to_string()),
                create_token(Token::Number, "10".to_string()),
                create_token(Token::RightBracket, "]".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::Global(vec![AstType::Variable(
                    Type::Int,
                    Structure::Array(vec![10]),
                    "a".to_string()
                ),])
            );
            assert_eq!(
                result.get_tree()[1],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Return(Box::new(
                        AstType::Factor(1)
                    ),)]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Char, "char".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Comma, ",".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Char, Structure::Identifier, "a".to_string()),
                        AstType::Variable(Type::Char, Structure::Identifier, "b".to_string()),
                        AstType::Return(Box::new(AstType::Factor(1)),)
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::IntPointer, "int*".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Comma, ",".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Pointer, "a".to_string()),
                        AstType::Variable(Type::Int, Structure::Pointer, "b".to_string()),
                        AstType::Return(Box::new(AstType::Factor(1)),)
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::CharPointer, "char*".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Comma, ",".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Char, Structure::Pointer, "a".to_string()),
                        AstType::Variable(Type::Char, Structure::Pointer, "b".to_string()),
                        AstType::Return(Box::new(AstType::Factor(1)),)
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Char, "char".to_string()),
                create_token(Token::Variable, "x".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::Global(vec![AstType::Variable(
                    Type::Int,
                    Structure::Identifier,
                    "a".to_string()
                ),
                AstType::Variable(
                    Type::Char,
                    Structure::Identifier,
                    "x".to_string()
                ),])
            );
            assert_eq!(
                result.get_tree()[1],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![AstType::Return(Box::new(
                        AstType::Factor(1)
                    ),)]))
                )
            );
        }
    }

    #[test]
    fn test_lvalue() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::IntPointer, "int*".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "10".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Assign(
                            Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Pointer,
                                "a".to_string()
                            )),
                            Box::new(AstType::Factor(10)),
                        ),
                        AstType::Return(Box::new(AstType::Factor(1)),)
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::IntPointer, "int*".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Multi, "*".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Plus, "+".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "10".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Pointer, "a".to_string()),
                        AstType::Assign(
                            Box::new(AstType::Indirect(Box::new(AstType::Plus(
                                Box::new(AstType::Variable(
                                    Type::Int,
                                    Structure::Pointer,
                                    "a".to_string()
                                )),
                                Box::new(AstType::Factor(2)),
                            )),)),
                            Box::new(AstType::Factor(10)),
                        ),
                        AstType::Return(Box::new(AstType::Factor(1)),)
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::LeftBracket, "[".to_string()),
                create_token(Token::Number, "10".to_string()),
                create_token(Token::RightBracket, "]".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::LeftBracket, "[".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::RightBracket, "]".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "10".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Array(vec![10]), "a".to_string()),
                        AstType::Assign(
                            Box::new(AstType::Indirect(Box::new(AstType::Plus(
                                Box::new(AstType::Variable(
                                    Type::Int,
                                    Structure::Array(vec![10]),
                                    "a".to_string()
                                )),
                                Box::new(AstType::Factor(2)),
                            )),)),
                            Box::new(AstType::Factor(10)),
                        ),
                        AstType::Return(Box::new(AstType::Factor(1)),)
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::LeftBracket, "[".to_string()),
                create_token(Token::Number, "10".to_string()),
                create_token(Token::RightBracket, "]".to_string()),
                create_token(Token::LeftBracket, "[".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::RightBracket, "]".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::LeftBracket, "[".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::RightBracket, "]".to_string()),
                create_token(Token::LeftBracket, "[".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::RightBracket, "]".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "10".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(
                            Type::Int,
                            Structure::Array(vec![10, 2]),
                            "a".to_string()
                        ),
                        AstType::Assign(
                            Box::new(AstType::Indirect(Box::new(AstType::Plus(
                                Box::new(AstType::Variable(
                                    Type::Int,
                                    Structure::Array(vec![10, 2]),
                                    "a".to_string()
                                )),
                                Box::new(AstType::Plus(
                                    Box::new(AstType::Multiple(
                                        Box::new(AstType::Factor(2)),
                                        Box::new(AstType::Factor(2)),
                                    )),
                                    Box::new(AstType::Factor(1)),
                                )),
                            )),)),
                            Box::new(AstType::Factor(10)),
                        ),
                        AstType::Return(Box::new(AstType::Factor(1)),)
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::LeftBracket, "[".to_string()),
                create_token(Token::Number, "10".to_string()),
                create_token(Token::RightBracket, "]".to_string()),
                create_token(Token::LeftBracket, "[".to_string()),
                create_token(Token::Number, "8".to_string()),
                create_token(Token::RightBracket, "]".to_string()),
                create_token(Token::LeftBracket, "[".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::RightBracket, "]".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::LeftBracket, "[".to_string()),
                create_token(Token::Number, "2".to_string()),
                create_token(Token::RightBracket, "]".to_string()),
                create_token(Token::LeftBracket, "[".to_string()),
                create_token(Token::Number, "4".to_string()),
                create_token(Token::RightBracket, "]".to_string()),
                create_token(Token::LeftBracket, "[".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::RightBracket, "]".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::Number, "10".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(
                            Type::Int,
                            Structure::Array(vec![10, 8, 2]),
                            "a".to_string()
                        ),
                        AstType::Assign(
                            Box::new(AstType::Indirect(Box::new(AstType::Plus(
                                Box::new(AstType::Variable(
                                    Type::Int,
                                    Structure::Array(vec![10, 8, 2]),
                                    "a".to_string()
                                )),
                                Box::new(AstType::Plus(
                                    Box::new(AstType::Multiple(
                                        Box::new(AstType::Factor(2)),
                                        Box::new(AstType::Factor(8)),
                                    )),
                                    Box::new(AstType::Plus(
                                        Box::new(AstType::Multiple(
                                            Box::new(AstType::Factor(4)),
                                            Box::new(AstType::Factor(2)),
                                        )),
                                        Box::new(AstType::Factor(1)),
                                    ))
                                )),
                            )),)),
                            Box::new(AstType::Factor(10)),
                        ),
                        AstType::Return(Box::new(AstType::Factor(1)),)
                    ]))
                )
            );
        }
    }

    #[test]
    fn test_post_inc_dec() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Inc, "++".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Identifier, "a".to_string()),
                        AstType::PostInc(Box::new(AstType::Variable(
                            Type::Int,
                            Structure::Identifier,
                            "a".to_string()
                        )),),
                        AstType::Return(Box::new(AstType::Factor(1)),)
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Dec, "--".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Identifier, "a".to_string()),
                        AstType::PostDec(Box::new(AstType::Variable(
                            Type::Int,
                            Structure::Identifier,
                            "a".to_string()
                        )),),
                        AstType::Return(Box::new(AstType::Factor(1)),)
                    ]))
                )
            );
        }
    }

    #[test]
    fn test_pre_inc_dec() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Inc, "++".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Identifier, "a".to_string()),
                        AstType::PreInc(Box::new(AstType::Variable(
                            Type::Int,
                            Structure::Identifier,
                            "a".to_string()
                        )),),
                        AstType::Return(Box::new(AstType::Factor(1)),)
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Dec, "--".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Identifier, "a".to_string()),
                        AstType::PreDec(Box::new(AstType::Variable(
                            Type::Int,
                            Structure::Identifier,
                            "a".to_string()
                        )),),
                        AstType::Return(Box::new(AstType::Factor(1)),)
                    ]))
                )
            );
        }
    }

    #[test]
    fn test_variable_char() {
        {
            let data = vec![
                create_token(Token::Char, "char".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Char, "char".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Char,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Char, Structure::Identifier, "a".to_string()),
                        AstType::Return(Box::new(AstType::Factor(1)),)
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Char, "char".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::LeftBracket, " [".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::RightBracket, "]".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Char, Structure::Array(vec![3]), "a".to_string()),
                        AstType::Return(Box::new(AstType::Factor(1)),)
                    ]))
                )
            );
        }
    }

    #[test]
    fn test_string_literal() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::CharPointer, "char*".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::StringLiteral, "testaaaa".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Char, Structure::Pointer, "a".to_string()),
                        AstType::Assign(
                            Box::new(AstType::Variable(
                                Type::Char,
                                Structure::Pointer,
                                "a".to_string()
                            )),
                            Box::new(AstType::StringLiteral("testaaaa".to_string(), 0))
                        ),
                        AstType::Return(Box::new(AstType::Factor(1)),)
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::CharPointer, "char*".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::StringLiteral, "test, aaaa".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Char, Structure::Pointer, "a".to_string()),
                        AstType::Assign(
                            Box::new(AstType::Variable(
                                Type::Char,
                                Structure::Pointer,
                                "a".to_string()
                            )),
                            Box::new(AstType::StringLiteral("test, aaaa".to_string(), 0))
                        ),
                        AstType::Return(Box::new(AstType::Factor(1)),)
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::CharPointer, "char*".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::StringLiteral, "test, aaaa".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::CharPointer, "char*".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::Assign, "=".to_string()),
                create_token(Token::StringLiteral, "test, bbbb".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::Number, "1".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Char, Structure::Pointer, "a".to_string()),
                        AstType::Assign(
                            Box::new(AstType::Variable(
                                Type::Char,
                                Structure::Pointer,
                                "a".to_string()
                            )),
                            Box::new(AstType::StringLiteral("test, aaaa".to_string(), 0))
                        ),
                        AstType::Variable(Type::Char, Structure::Pointer, "b".to_string()),
                        AstType::Assign(
                            Box::new(AstType::Variable(
                                Type::Char,
                                Structure::Pointer,
                                "b".to_string()
                            )),
                            Box::new(AstType::StringLiteral("test, bbbb".to_string(), 1))
                        ),
                        AstType::Return(Box::new(AstType::Factor(1)),)
                    ]))
                )
            );
        }
    }

    #[test]
    fn test_sizeof() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Char, "char".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::SizeOf, "sizeof".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::RightParen, "(".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "end".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Char, Structure::Identifier, "a".to_string()),
                        AstType::Return(Box::new(AstType::SizeOf(1)),)
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::SizeOf, "sizeof".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::RightParen, "(".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "end".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Return(Box::new(AstType::SizeOf(4)),)
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::SizeOf, "sizeof".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::Char, "char".to_string()),
                create_token(Token::RightParen, "(".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "end".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Return(Box::new(AstType::SizeOf(1)),)
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::SizeOf, "sizeof".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::IntPointer, "int*".to_string()),
                create_token(Token::RightParen, "(".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "end".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Return(Box::new(AstType::SizeOf(8)),)
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::SizeOf, "sizeof".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::CharPointer, "char*".to_string()),
                create_token(Token::RightParen, "(".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "end".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Return(Box::new(AstType::SizeOf(8)),)
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::SizeOf, "sizeof".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::RightParen, "(".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "end".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Identifier, "a".to_string()),
                        AstType::Return(Box::new(AstType::SizeOf(4)),)
                    ]))
                )
            );
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::LeftBracket, " [".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::RightBracket, "]".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Return, "return".to_string()),
                create_token(Token::SizeOf, "sizeof".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(Type::Int, Structure::Array(vec![3]), "a".to_string()),
                        AstType::Return(Box::new(AstType::SizeOf(24)),)
                    ]))
                )
            );
        }
    }

    #[test]
    fn test_plus_assign() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::PlusAssign, "+=".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(
                            Type::Int,
                            Structure::Identifier,
                            "a".to_string()
                        ),
                        AstType::PlusAssign(
                            Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Identifier,
                                "a".to_string()
                            )),
                            Box::new(AstType::Factor(3))
                        )
                    ])),
                )
            )
        }
    }

    #[test]
    fn test_minus_assign() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::MinusAssign, "-=".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(
                            Type::Int,
                            Structure::Identifier,
                            "a".to_string()
                        ),
                        AstType::MinusAssign(
                            Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Identifier,
                                "a".to_string()
                            )),
                            Box::new(AstType::Factor(3))
                        )
                    ,])),
                )
            )
        }
    }

    #[test]
    fn test_multiple_assign() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::MultipleAssign, "*=".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(
                            Type::Int,
                            Structure::Identifier,
                            "a".to_string()
                        ),
                        AstType::MultipleAssign(
                            Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Identifier,
                                "a".to_string()
                            )),
                            Box::new(AstType::Factor(3))
                        )
                    ,])),
                )
            )
        }
    }

    #[test]
    fn test_division_assign() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::DivisionAssign, "/=".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(
                            Type::Int,
                            Structure::Identifier,
                            "a".to_string()
                        ),
                        AstType::DivisionAssign(
                            Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Identifier,
                                "a".to_string()
                            )),
                            Box::new(AstType::Factor(3))
                        )
                    ,])),
                )
            )
        }
    }

    #[test]
    fn test_remainder_assign() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::RemainderAssign, "%=".to_string()),
                create_token(Token::Number, "3".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(AstType::Statement(vec![
                        AstType::Variable(
                            Type::Int,
                            Structure::Identifier,
                            "a".to_string()
                        ),
                        AstType::RemainderAssign(
                            Box::new(AstType::Variable(
                                Type::Int,
                                Structure::Identifier,
                                "a".to_string()
                            )),
                            Box::new(AstType::Factor(3))
                        )
                    ,])),
                )
            )
        }
    }

    #[test]
    fn test_struct() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Struct, "struct".to_string()),
                create_token(Token::Variable, "Test".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(
                        AstType::Statement(vec![
                            AstType::Struct(
                                Box::new(AstType::Variable(Type::Struct("Test".to_string()), Structure::Struct, "Test".to_string())),
                                vec![]
                            )
                        ])
                    ),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Struct, "struct".to_string()),
                create_token(Token::Variable, "Test".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(
                        AstType::Statement(vec![
                            AstType::Struct(
                                Box::new(AstType::Variable(Type::Struct("Test".to_string()), Structure::Struct, "Test".to_string())),
                                vec![
                                    AstType::Variable(
                                        Type::Int,
                                        Structure::Identifier,
                                        "a".to_string()
                                    )
                                ]
                            )
                        ])
                    ),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Struct, "struct".to_string()),
                create_token(Token::Variable, "Test".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(
                        AstType::Statement(vec![
                            AstType::Struct(
                                Box::new(AstType::Variable(Type::Struct("Test".to_string()), Structure::Struct, "Test".to_string())),
                                vec![
                                    AstType::Variable(
                                        Type::Int,
                                        Structure::Identifier,
                                        "a".to_string()
                                    ),
                                    AstType::Variable(
                                        Type::Int,
                                        Structure::Identifier,
                                        "b".to_string()
                                    )
                                ]
                            )
                        ])
                    ),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Struct, "struct".to_string()),
                create_token(Token::Variable, "Test".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Char, "char".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(
                        AstType::Statement(vec![
                            AstType::Struct(
                                Box::new(AstType::Variable(Type::Struct("Test".to_string()), Structure::Struct, "Test".to_string())),
                                vec![
                                    AstType::Variable(
                                        Type::Int,
                                        Structure::Identifier,
                                        "a".to_string()
                                    ),
                                    AstType::Variable(
                                        Type::Char,
                                        Structure::Identifier,
                                        "b".to_string()
                                    )
                                ]
                            )
                        ])
                    ),
                )
            )
        }
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Struct, "struct".to_string()),
                create_token(Token::Variable, "Test".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "a".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Char, "char".to_string()),
                create_token(Token::Variable, "b".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::Global(vec![
                    AstType::Variable(
                        Type::Int,
                        Structure::Identifier,
                        "a".to_string()
                    ),
                    AstType::Struct(
                        Box::new(AstType::Variable(Type::Struct("Test".to_string()), Structure::Struct, "Test".to_string())),
                        vec![
                            AstType::Variable(
                                Type::Int,
                                Structure::Identifier,
                                "a".to_string()
                            ),
                            AstType::Variable(
                                Type::Char,
                                Structure::Identifier,
                                "b".to_string()
                            )
                        ]
                    )
                ])
            );
            assert_eq!(
                result.get_tree()[1],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(
                        AstType::Statement(vec![])
                    ),
                )
            )
        }
    }

    #[test]
    fn test_struct_val() {
        {
            let data = vec![
                create_token(Token::Int, "int".to_string()),
                create_token(Token::Variable, "main".to_string()),
                create_token(Token::LeftParen, "(".to_string()),
                create_token(Token::RightParen, ")".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::Struct, "struct".to_string()),
                create_token(Token::Variable, "Test".to_string()),
                create_token(Token::LeftBrace, "{".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::Struct, "struct".to_string()),
                create_token(Token::Variable, "Test".to_string()),
                create_token(Token::Variable, "test".to_string()),
                create_token(Token::SemiColon, ";".to_string()),
                create_token(Token::RightBrace, "}".to_string()),
                create_token(Token::End, "End".to_string()),
            ];
            let mut ast = AstGen::new(&data);
            let result = ast.parse();

            // 期待値確認.
            assert_eq!(
                result.get_tree()[0],
                AstType::FuncDef(
                    Type::Int,
                    Structure::Identifier,
                    "main".to_string(),
                    Box::new(AstType::Argment(vec![])),
                    Box::new(
                        AstType::Statement(vec![
                            AstType::Struct(
                                Box::new(AstType::Variable(Type::Struct("Test".to_string()), Structure::Struct, "Test".to_string())),
                                vec![]
                            ),
                            AstType::Variable(
                                Type::Struct("Test".to_string()), Structure::Struct, "test".to_string()
                            ),
                        ])
                    ),
                )
            )
        }
    }
}
