use arch::Generator;
use arch::{x64::X64};
use arch::{x64_mac::X64Mac};
use ast::{AstTree, AstType};
use config::Config;
use std::process;
use symbol::{Scope, Structure, Symbol, SymbolTable, Type};

#[doc = "ラベル管理"]
struct Label {
    label_no: usize,
    continue_labels: Vec<usize>,
    break_labels: Vec<usize>,
    return_label: usize,
}

impl Label {
    // コンストラクタ.
    pub fn new() -> Self {
        Label {
            label_no: 0,
            continue_labels: vec![],
            break_labels: vec![],
            return_label: 0,
        }
    }

    // ラベル番号インクリメント.
    pub fn next_label(&mut self) -> usize {
        self.label_no += 1;
        self.label_no
    }

    // returnラベル取得.
    pub fn next_return_label(&mut self) -> usize {
        self.return_label = self.next_label();
        self.return_label
    }
    pub fn get_return_label(&self) -> usize {
        self.return_label
    }

    // continueラベル追加.
    pub fn push_continue(&mut self, no: usize) {
        self.continue_labels.push(no);
    }
    // continueラベルpop.
    pub fn pop_continue(&mut self) -> Option<usize> {
        self.continue_labels.pop()
    }
    // continueラベル削除.
    pub fn remove_continue(&mut self, no: usize) {
        self.continue_labels = self
            .continue_labels
            .iter()
            .cloned()
            .filter(|d| *d != no)
            .collect();
    }
    // breakラベル追加.
    pub fn push_break(&mut self, no: usize) {
        self.break_labels.push(no);
    }
    // breakラベルpop.
    pub fn pop_break(&mut self) -> Option<usize> {
        self.break_labels.pop()
    }
    // breakラベル削除.
    pub fn remove_break(&mut self, no: usize) {
        self.break_labels = self
            .break_labels
            .iter()
            .cloned()
            .filter(|d| *d != no)
            .collect();
    }
}

// 関数引数レジスタ.
const REGS: &[&str] = &["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

#[doc = "アセンブラ生成部"]
pub struct Asm<'a> {
    inst: String,
    const_literal: String,
    sym_table: &'a SymbolTable,
    cur_scope: Scope,
    label: Label,
}

impl<'a> Asm<'a> {
    // コンストラクタ.
    pub fn new(table: &'a SymbolTable) -> Asm<'a> {
        Asm {
            inst: "".to_string(),
            const_literal: "".to_string(),
            label: Label::new(),
            sym_table: table,
            cur_scope: Scope::Unknown,
        }
    }

    // アセンブラ生成部取得
    fn gen_asm(&self) -> Box<dyn Generator> {
        if Config::is_mac() {
            Box::new(X64Mac)
        } else {
            Box::new(X64)
        }
    }

    // アセンブラ取得
    pub fn get_inst(&self) -> String {
        // 定数領域と結合
        format!("{}{}", self.const_literal, self.inst)
    }

    // アセンブラ生成開始.
    pub fn exec(&mut self, tree: &AstTree) {
        tree.get_tree().iter().for_each(|a| self.generate(a));
    }

    // 現在スコープ切り替え
    fn switch_scope(&mut self, scope: Scope) {
        self.cur_scope = scope;
    }

    // アセンブラ生成.
    fn generate(&mut self, ast: &AstType) {
        match *ast {
            AstType::Global(ref a) => {
                self.switch_scope(Scope::Global);
                self.generate_global(a);
            }
            AstType::FuncDef(ref t, ref _s, ref a, ref b, ref c) => {
                self.switch_scope(Scope::Local(a.clone()));
                self.generate_funcdef(t, a, b, c);
            }
            AstType::FuncCall(ref a, ref b) => self.generate_call_func(a, b),
            AstType::Statement(_) => self.generate_statement(ast),
            AstType::While(ref a, ref b) => self.generate_statement_while(a, b),
            AstType::Do(ref a, ref b) => self.generate_statement_do(a, b),
            AstType::If(ref a, ref b, ref c) => self.generate_statement_if(a, b, c),
            AstType::For(ref a, ref b, ref c, ref d) => self.generate_statement_for(a, b, c, d),
            AstType::Continue() => self.generate_statement_continue(),
            AstType::Break() => self.generate_statement_break(),
            AstType::Return(ref a) => self.generate_statement_return(a),
            AstType::SizeOf(a) => self.generate_sizeof(a),
            AstType::Factor(a) => self.generate_factor(a),
            AstType::LogicalAnd(ref a, ref b) => self.generate_logical_and(a, b),
            AstType::LogicalOr(ref a, ref b) => self.generate_logical_or(a, b),
            AstType::Condition(ref a, ref b, ref c) => self.generate_condition(a, b, c),
            AstType::UnPlus(ref a) => self.generate_unplus(a),
            AstType::UnMinus(ref a) => self.generate_unminus(a),
            AstType::Not(ref a) => self.generate_not(a),
            AstType::BitReverse(ref a) => self.generate_bit_reverse(a),
            AstType::Assign(ref a, ref b) => self.generate_assign(a, b),
            AstType::PlusAssign(ref a, ref b) => self.generate_plus_assign(a, b),
            AstType::MinusAssign(ref a, ref b) => self.generate_minus_assign(a, b),
            AstType::MultipleAssign(ref a, ref b) => self.generate_multiple_assign(a, b),
            AstType::DivisionAssign(ref a, ref b) => self.generate_division_assign(a, b),
            AstType::RemainderAssign(ref a, ref b) => self.generate_remainder_assign(a, b),
            AstType::Variable(_, _, _) => self.generate_variable(ast),
            AstType::PreInc(ref a) => self.generate_pre_inc(a),
            AstType::PreDec(ref a) => self.generate_pre_dec(a),
            AstType::PostInc(ref a) => self.generate_post_inc(a),
            AstType::PostDec(ref a) => self.generate_post_dec(a),
            AstType::Plus(ref a, ref b) => self.generate_plus(a, b),
            AstType::Minus(ref a, ref b) => self.generate_minus(a, b),
            AstType::Multiple(ref a, ref b)
            | AstType::Division(ref a, ref b)
            | AstType::Remainder(ref a, ref b)
            | AstType::Equal(ref a, ref b)
            | AstType::NotEqual(ref a, ref b)
            | AstType::LessThan(ref a, ref b)
            | AstType::GreaterThan(ref a, ref b)
            | AstType::LessThanEqual(ref a, ref b)
            | AstType::GreaterThanEqual(ref a, ref b)
            | AstType::LeftShift(ref a, ref b)
            | AstType::RightShift(ref a, ref b)
            | AstType::BitAnd(ref a, ref b)
            | AstType::BitOr(ref a, ref b)
            | AstType::BitXor(ref a, ref b) => self.generate_operator(ast, a, b),
            AstType::Address(ref a) => self.generate_address(a),
            AstType::Indirect(ref a) => self.generate_indirect(a),
            AstType::StringLiteral(ref s, ref i) => {
                self.generate_string_literal(&AstType::StringLiteral(s.to_string(), *i));
                self.generate_string(s, *i);
            }
            AstType::Struct(ref _a, ref _b) => {}, // 構造体定義のみなので、現状は何もしない
            _ => panic!("{} {}: not support expression {:?}", file!(), line!(), ast),
        }
    }

    // グローバル変数代入
    fn generate_global_assign(&mut self, a: &AstType, b: &AstType) {
        // 左辺が変数、右辺は数字をサポート
        match b {
            AstType::Factor(i) => match a {
                AstType::Variable(ref t, _, ref name) => {
                    self.inst = format!("{}{}:\n", self.inst, name);
                    self.inst = match t {
                        Type::Int =>  format!("{}  .long {}\n", self.inst, i),
                        Type::Char => format!("{}  .byte {}\n", self.inst, i),
                        _ => panic!("{}{}: cannot support type {:?}", file!(), line!(), t)
                    }
                }
                _ => panic!("{}{}: cannot support AstType {:?}", file!(), line!(), b)
            }
            _ => panic!("{}{}: cannot support AstType {:?}", file!(), line!(), a)
        }
    }

    // グローバル変数定義
    fn generate_global(&mut self, a: &[AstType]) {
        self.inst = format!("{}{}", self.inst, "  .data\n");
        a.iter().for_each(|d| {
            match d {
                AstType::Assign(ref a, ref b) => self.generate_global_assign(a, b),
                AstType::Variable(_, _, ref name) => {
                    self.inst = format!("{}{}:\n", self.inst, name);
                    self.inst = format!("{}  .zero 8\n", self.inst);
                }
                AstType::Struct(_, _) => {}, // 構造体定義のみなのでSKIP
                _ => panic!("{}{}: cannot support AstType {:?}", file!(), line!(), d)
            }
        });
    }

    // 関数定義.
    fn generate_funcdef(&mut self, _t: &Type, a: &str, b: &AstType, c: &AstType) {
        // return文のラベルを生成.
        let return_label = self.label.next_return_label();

        self.generate_func_start(a);
        self.generate_func_args(b);
        self.generate_statement(c);
        self.generate_label_inst(return_label);
        self.generate_func_end();
    }

    // statement生成.
    fn generate_statement(&mut self, a: &AstType) {
        // 各AstTypeを処理.
        match *a {
            AstType::Statement(ref s) => s.iter().for_each(|ast| {
                self.generate(ast);
                if ast.is_expr() {
                    self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rax"));
                }
            }),
            _ => panic!("{} {}: not support expr", file!(), line!()),
        }
    }

    // 関数開始アセンブラ出力.
    fn generate_func_start(&mut self, a: &str) {
        // スタート部分設定.
        let mut start = if a == "main" {
            format!("  .text\n.global {}\n", self.generate_func_symbol(a))
        } else {
            "  .text\n".to_string()
        };

        // 16バイトアライメント
        let mut pos = self.sym_table.size(&Scope::Local(a.to_string())) ;
        pos = (pos / 16) * 16 + 16;
        start = format!("{}{}{}:\n", self.inst, start, self.generate_func_symbol(a));
        start = format!(
            "{}{}{}{}",
            start,
            self.gen_asm().push("rbp"),
            self.gen_asm().mov("rsp", "rbp"),
            self.gen_asm().sub_imm(pos, "rsp")
        );
        self.inst = start;
    }

    // 関数終了部分アセンブラ生成
    fn generate_func_end(&mut self) {
        self.inst = format!(
            "{}{}{}",
            self.inst,
            self.gen_asm().leave(),
            self.gen_asm().ret()
        );
    }

    // 関数引数生成.
    fn generate_func_args(&mut self, a: &AstType) {
        // レジスタからスタックへ引数を移動(SPを8バイトずつ移動しながら).
        let st = 8;
        match *a {
            AstType::Argment(ref args) => {
                args.iter().zip(REGS.iter()).fold(st, |p, d| {
                    match d.0 {
                        AstType::Variable(_, s, _) if *s == Structure::Pointer => {
                            self.inst = format!(
                                "{}{}",
                                self.inst,
                                self.gen_asm().mov_dst(&d.1, "rbp", -(p as i64))
                            );
                        }
                        _ => {
                            self.inst = format!(
                                "{}{}{}",
                                self.inst,
                                self.gen_asm().mov(&d.1, "rax"),
                                self.gen_asm().mov_dst("rax", "rbp", -(p as i64))
                            );
                        }
                    };
                    p + 8
                });
            }
            _ => panic!("{} {}: not support expr {:?}", file!(), line!(), a),
        }
    }

    // if statement生成.
    fn generate_statement_if(&mut self, a: &AstType, b: &AstType, c: &Option<AstType>) {
        let label_end = self.label.next_label();

        // 条件式部分生成.
        self.generate(a);
        self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rax"));
        self.generate_cmp_inst(1, "rax"); // 等しい場合は、1に設定されている.

        // elseブロック生成.
        match c {
            Some(e) => {
                // if条件が満たされているとき、ifラベルへ
                let label_if = self.label.next_label();
                self.generate_je_inst(label_if);

                // elseブロック生成.
                // block部はAstType::Statementなので、演算結果に対するスタック操作は行わない.
                self.generate(e);
                self.generate_jmp_inst(label_end);

                // ifブロック部生成.
                // block部はAstType::Statementなので、演算結果に対するスタック操作は行わない.
                self.generate_label_inst(label_if);
                self.generate(b);
                self.generate_jmp_inst(label_end);
            }
            _ => {
                // if条件が満たされていない場合、endラベルへ
                self.generate_jne_inst(label_end);

                // ifブロック部生成.
                // block部はAstType::Statementなので、演算結果に対するスタック操作は行わない.
                self.generate(b);
            }
        }
        // 終端ラベル
        self.generate_label_inst(label_end);
    }

    // while statement生成.
    fn generate_statement_while(&mut self, a: &AstType, b: &AstType) {
        let label_begin = self.label.next_label();
        let label_end = self.label.next_label();

        // continue/breakラベル生成.
        self.label.push_continue(label_begin);
        self.label.push_break(label_end);

        // condition部生成.
        self.generate_label_inst(label_begin);
        self.generate(a);
        // conditionが偽であれば、ブロック終端へジャンプ.
        self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rax"));
        self.generate_cmp_inst(0, "rax");
        self.generate_je_inst(label_end);

        // ブロック部生成.
        // block部はAstType::Statementなので、演算結果に対するスタック操作は行わない.
        self.generate(b);
        self.generate_jmp_inst(label_begin);

        // endラベル.
        self.generate_label_inst(label_end);

        // 生成したcontinue/breakラベルを除去.
        self.label.remove_continue(label_begin);
        self.label.remove_break(label_end);
    }

    // do-while statement生成.
    fn generate_statement_do(&mut self, a: &AstType, b: &AstType) {
        let label_begin = self.label.next_label();
        let label_condition = self.label.next_label();
        let label_end = self.label.next_label();

        // continue/breakラベル生成.
        self.label.push_continue(label_condition);
        self.label.push_break(label_end);

        // ブロック部生成.
        self.generate_label_inst(label_begin);
        self.generate(a);

        // condition部生成.
        self.generate_label_inst(label_condition);
        self.generate(b);
        // conditionが真であれば、ブロック先頭へジャンプ.
        self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rax"));
        self.generate_cmp_inst(0, "rax");

        self.generate_jne_inst(label_begin);
        self.generate_label_inst(label_end);

        // 生成したcontinue/breakラベルを除去.
        self.label.remove_continue(label_condition);
        self.label.remove_break(label_end);
    }

    // for statement生成.
    fn generate_statement_for(
        &mut self,
        a: &Option<AstType>,
        b: &Option<AstType>,
        c: &Option<AstType>,
        d: &AstType,
    ) {
        let label_begin = self.label.next_label();
        let label_continue = self.label.next_label();
        let label_end = self.label.next_label();

        // continue/breakラベル生成.
        self.label.push_continue(label_continue);
        self.label.push_break(label_end);

        // 初期条件.
        if let Some(init) = a {
            self.generate(init);
            self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rax"));
        }
        self.generate_label_inst(label_begin);

        // 終了条件.
        if let Some(cond) = b {
            self.generate(cond);
            self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rax"));
            self.generate_cmp_inst(0, "rax");
            self.generate_je_inst(label_end);
        }

        // ブロック部.
        self.generate(d);
        self.generate_label_inst(label_continue);

        // 変数変化部分生成
        if let Some(end) = c {
            self.generate(end);
            self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rax"));
        }
        self.generate_jmp_inst(label_begin);
        self.generate_label_inst(label_end);

        // 生成したcontinue/breakラベルを除去.
        self.label.remove_continue(label_continue);
        self.label.remove_break(label_end);
    }

    // continue文生成.
    fn generate_statement_continue(&mut self) {
        let label = self.label.pop_continue();
        let no = label.expect("asm.rs(generate_statement_continue): invalid continue label");
        self.generate_jmp_inst(no);
    }

    // break文生成.
    fn generate_statement_break(&mut self) {
        let label = self.label.pop_break();
        let no = label.expect("asm.rs(generate_statement_break): invalid continue label");
        self.generate_jmp_inst(no);
    }

    // return statement.
    fn generate_statement_return(&mut self, a: &AstType) {
        self.generate(a);
        if a.is_expr() {
            self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rax"));
        }
        let label_no = self.label.get_return_label();
        self.generate_jmp_inst(label_no);
    }

    // assign indirect
    fn generate_assign_indirect(&mut self, a: &AstType, b: &AstType) {
        self.generate(a);
        self.generate(b);
        self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rax"));
        self.inst = format!(
            "{}{}{}",
            self.inst,
            self.gen_asm().pop("rcx"),
            self.gen_asm().mov_dst("rax", "rcx", 0)
        );
        self.inst = format!("{}{}", self.inst, self.gen_asm().push("rax"));
    }

    // assign生成.
    fn generate_assign(&mut self, a: &AstType, b: &AstType) {
        match *a {
            AstType::Variable(ref t, ref s, _) => {
                self.generate_lvalue_address(a);
                self.generate(b);
                self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rcx"));
                self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rax"));

                // ポインタは64bitで転送
                match s {
                    Structure::Pointer => {
                        self.inst = format!("{}{}", self.inst, self.gen_asm().mov_dst("rcx", "rax", 0));
                    }
                    _ => {
                        // 型に応じた転送サイズを考慮
                        match t {
                            Type::Char => {
                                self.inst = format!("{}{}", self.inst, self.gen_asm().movb_dst("cl", "rax", 0));
                            }
                            _ =>  {
                                self.inst = format!("{}{}", self.inst, self.gen_asm().mov_dst("rcx", "rax", 0));
                            }
                        }
                    }
                }
                self.inst = format!("{}{}", self.inst, self.gen_asm().push("rcx"));
            }
            AstType::Indirect(ref a) => self.generate_assign_indirect(a, b),
            _ => self.generate(b),
        }
    }

    // plus assign生成.
    fn generate_plus_assign(&mut self, a: &AstType, b: &AstType) {
        match a {
            AstType::Variable(_, _, ref name) => {
                self.generate_lvalue_address(a);
                self.generate(b);
                self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rax"));
                self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rcx"));
                self.inst = format!("{}{}", self.inst, self.gen_asm().add_src("rcx", "rax", 0));

                // 型に応じた転送サイズを考慮
                let sym = self.get_var_symbol(name);
                match sym.t {
                    Type::Char => {
                        self.inst = format!("{}{}", self.inst, self.gen_asm().movb_dst("al", "rcx", 0));
                    }
                    _ =>  {
                        self.inst = format!("{}{}", self.inst, self.gen_asm().mov_dst("rax", "rcx", 0));
                    }
                }
            }
            _ => panic!("{} {}: cannot support AstType {:?}", file!(), line!(), a)
        }
    }

    // minus assign生成.
    fn generate_minus_assign(&mut self, a: &AstType, b: &AstType) {
        match a {
            AstType::Variable(_, _, ref name) => {
                self.generate_lvalue_address(a);
                self.generate(b);
                self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rax"));
                self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rcx"));
                self.inst = format!("{}{}", self.inst, self.gen_asm().mov_src("rcx", "rdx", 0));
                self.inst = format!("{}{}", self.inst, self.gen_asm().sub("rax", "rdx"));

                // 型に応じた転送サイズを考慮
                let sym = self.get_var_symbol(name);
                match sym.t {
                    Type::Char => {
                        self.inst = format!("{}{}", self.inst, self.gen_asm().movb_dst("dl", "rcx", 0));
                    }
                    _ =>  {
                        self.inst = format!("{}{}", self.inst, self.gen_asm().mov_dst("rdx", "rcx", 0));
                    }
                }
            }
            _ => panic!("{} {}: cannot support AstType {:?}", file!(), line!(), a)
        }
    }

    // multiple assign生成.
    fn generate_multiple_assign(&mut self, a: &AstType, b: &AstType) {
        match a {
            AstType::Variable(_, _, ref name) => {
                self.generate_lvalue_address(a);
                self.generate(b);
                self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rax"));
                self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rcx"));
                self.inst = format!("{}{}", self.inst, self.gen_asm().mov_src("rcx", "rdx", 0));
                self.inst = format!("{}{}", self.inst, self.gen_asm().mul("rdx"));

                // 型に応じた転送サイズを考慮
                let sym = self.get_var_symbol(name);
                match sym.t {
                    Type::Char => {
                        self.inst = format!("{}{}", self.inst, self.gen_asm().movb_dst("al", "rcx", 0));
                    }
                    _ =>  {
                        self.inst = format!("{}{}", self.inst, self.gen_asm().mov_dst("rax", "rcx", 0));
                    }
                }
            }
            _ => panic!("{} {}: cannot support AstType {:?}", file!(), line!(), a)
        }
    }

    // division assign生成.
    fn generate_division_assign(&mut self, a: &AstType, b: &AstType) {
        match a {
            AstType::Variable(_, _, ref name) => {
                self.generate_lvalue_address(a);
                self.generate(b);
                self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rcx"));
                self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rbx"));
                self.inst = format!("{}{}", self.inst, self.gen_asm().mov_src("rbx", "rax", 0));
                self.inst = format!("{}{}", self.inst, self.gen_asm().bit_division());

                // 型に応じた転送サイズを考慮
                let sym = self.get_var_symbol(name);
                match sym.t {
                    Type::Char => {
                        self.inst = format!("{}{}", self.inst, self.gen_asm().movb_dst("al", "rbx", 0));
                    }
                    _ =>  {
                        self.inst = format!("{}{}", self.inst, self.gen_asm().mov_dst("rax", "rbx", 0));
                    }
                }
            }
            _ => panic!("{} {}: cannot support AstType {:?}", file!(), line!(), a)
        }
    }

    // remainder assign生成.
    fn generate_remainder_assign(&mut self, a: &AstType, b: &AstType) {
        match a {
            AstType::Variable(_, _, ref name) => {
                self.generate_lvalue_address(a);
                self.generate(b);
                self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rcx"));
                self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rbx"));
                self.inst = format!("{}{}", self.inst, self.gen_asm().mov_src("rbx", "rax", 0));
                self.inst = format!("{}{}", self.inst, self.gen_asm().bit_division());

                // 型に応じた転送サイズを考慮
                let sym = self.get_var_symbol(name);
                match sym.t {
                    Type::Char => {
                        self.inst = format!("{}{}", self.inst, self.gen_asm().movb_dst("dl", "rbx", 0));
                    }
                    _ =>  {
                        self.inst = format!("{}{}", self.inst, self.gen_asm().mov_dst("rdx", "rbx", 0));
                    }
                }
            }
            _ => panic!("{} {}: cannot support AstType {:?}", file!(), line!(), a)
        }
    }

    // 型や構造を判断し、variable生成
    fn generate_variable_by_strt(&mut self, sym: &Symbol) {
        match sym.strt {
            Structure::Pointer => {
                self.inst = format!("{}{}", self.inst, self.gen_asm().movq_src("rcx", "rax", 0));
            }
            Structure::Array(_) => {
                self.inst = format!("{}{}", self.inst, self.gen_asm().movq("rcx", "rax"));
            }
            Structure::Identifier => {
                match sym.t {
                    Type::Int => {
                        self.inst = format!("{}{}", self.inst, self.gen_asm().movl_src("rcx", "eax", 0));
                    }
                    Type::Char => {
                        self.inst = format!("{}{}", self.inst, self.gen_asm().movsbl_src("rcx", "eax", 0));
                    }
                    _ => panic!("{}{}: cannot support type: {:?}", file!(), line!(), sym.t)
                }
            }
            Structure::Struct => {
                // 変数自体の割当は未実装
            },
            _ => panic!("{}{}: cannot support structure: {:?}", file!(), line!(), sym.strt)
        }
    }

    // variable生成.
    fn generate_variable(&mut self, a: &AstType) {
        self.generate_lvalue_address(a);
        match a {
            AstType::Variable(_, _, ref name) => {
                self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rcx"));
                let sym = self.get_var_symbol(name);
                self.generate_variable_by_strt(&sym);
            }
            _ => panic!("{}{}: cannot support AstType: {:?}", file!(), line!(), a)
        }
        self.inst = format!("{}{}", self.inst, self.gen_asm().push("rax"));
    }

    // 関数コール生成.
    fn generate_call_func(&mut self, lhs: &AstType, rhs: &AstType) {
        match *lhs {
            // 関数名.
            AstType::Variable(_, _, ref n) if self.sym_table.search(&Scope::Func, n).is_some() => {
                match *rhs {
                    AstType::Argment(ref v) => {
                        // 各引数を評価（スタックに積むので、逆順で積んでいく）.
                        v.iter().rev().for_each(|d| self.generate(d));

                        // 関数引数をスタックからレジスタへ.
                        v.iter().zip(REGS.iter()).for_each(|d| match d.0 {
                            AstType::Variable(_, s, _) if *s == Structure::Pointer => {
                                self.inst = format!("{}{}", self.inst, self.gen_asm().pop(&d.1));
                            }
                            _ => {
                                self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rax"));
                                self.inst =
                                    format!("{}{}", self.inst, self.gen_asm().mov("rax", &d.1));
                            }
                        });
                    }
                    _ => panic!("{} {}: Not Function Argment", file!(), line!()),
                }

                self.inst = format!(
                    "{}{}",
                    self.inst,
                    self.gen_asm().call(&self.generate_func_symbol(n))
                );
                self.inst = format!("{}{}", self.inst, self.gen_asm().push("rax"));
            }
            _ => panic!("{} {}: Not Exists Function name", file!(), line!()),
        }
    }

    // 関数シンボル生成.
    fn generate_func_symbol(&self, s: &str) -> String {
        if Config::is_mac() {
            format!("_{}", s)
        } else {
            s.to_string()
        }
    }

    // bit反転演算子生成.
    fn generate_bit_reverse(&mut self, a: &AstType) {
        self.generate(a);
        self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rax"));
        self.inst = format!("{}{}", self.inst, self.gen_asm().not("rax"));
        self.inst = format!("{}{}", self.inst, self.gen_asm().push("rax"));
    }

    // Not演算子生成.
    fn generate_not(&mut self, a: &AstType) {
        self.generate(a);
        self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rax"));
        self.generate_cmp_inst(0, "rax");
        self.inst = format!("{}{}", self.inst, self.gen_asm().set("al"));
        self.inst = format!("{}{}", self.inst, self.gen_asm().movz("al", "rax"));
        self.inst = format!("{}{}", self.inst, self.gen_asm().push("rax"));
    }

    // マイナス単項演算子生成.
    fn generate_unminus(&mut self, a: &AstType) {
        self.generate(a);
        self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rax"));
        self.inst = format!("{}{}", self.inst, self.gen_asm().neg("rax"));
        self.inst = format!("{}{}", self.inst, self.gen_asm().push("rax"));
    }

    // プラス単項演算子生成.
    fn generate_unplus(&mut self, a: &AstType) {
        self.generate(a);
    }

    // 三項演算子生成.
    fn generate_condition(&mut self, a: &AstType, b: &AstType, c: &AstType) {
        let label_false = self.label.next_label();
        let label_end = self.label.next_label();

        self.generate(a);
        self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rax"));
        self.generate_cmp_inst(0, "rax");
        self.generate_je_inst(label_false);

        self.generate(b);
        self.generate_jmp_inst(label_end);
        self.generate_label_inst(label_false);

        self.generate(c);
        self.generate_label_inst(label_end);
    }

    // &&演算子生成.
    fn generate_logical_and(&mut self, a: &AstType, b: &AstType) {
        let label_false = self.label.next_label();
        let label_end = self.label.next_label();

        self.generate(a);
        self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rax"));
        self.generate_cmp_inst(0, "rax");
        self.generate_je_inst(label_false);
        self.generate(b);
        self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rax"));
        self.generate_cmp_inst(0, "rax");
        self.generate_je_inst(label_false);

        self.inst = format!("{}{}", self.inst, self.gen_asm().mov_imm("rax", 1));
        self.generate_jmp_inst(label_end);
        self.generate_label_inst(label_false);
        self.inst = format!("{}{}", self.inst, self.gen_asm().mov_imm("rax", 0));
        self.generate_label_inst(label_end);
        self.inst = format!("{}{}", self.inst, self.gen_asm().push("rax"));
    }

    // ||演算子生成.
    fn generate_logical_or(&mut self, a: &AstType, b: &AstType) {
        let label_true = self.label.next_label();
        let label_end = self.label.next_label();

        self.generate(a);
        self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rax"));
        self.generate_cmp_inst(0, "rax");
        self.generate_jne_inst(label_true);
        self.generate(b);
        self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rax"));
        self.generate_cmp_inst(0, "rax");
        self.generate_jne_inst(label_true);

        self.inst = format!("{}{}", self.inst, self.gen_asm().mov_imm("rax", 0));
        self.generate_jmp_inst(label_end);
        self.generate_label_inst(label_true);
        self.inst = format!("{}{}", self.inst, self.gen_asm().mov_imm("rax", 1));
        self.generate_label_inst(label_end);
        self.inst = format!("{}{}", self.inst, self.gen_asm().push("rax"));
    }

    // 数値生成.
    fn generate_factor(&mut self, a: i64) {
        // 数値.
        self.inst = format!("{}{}", self.inst, self.gen_asm().mov_imm("rax", a));
        self.inst = format!("{}{}", self.inst, self.gen_asm().push("rax"));
    }

    // sizeof演算子.
    fn generate_sizeof(&mut self, a: usize) {
        // 数値.
        self.inst = format!("{}{}", self.inst, self.gen_asm().mov_imm("rax", a as i64));
        self.inst = format!("{}{}", self.inst, self.gen_asm().push("rax"));
    }

    // シンボル情報取得
    fn get_var_symbol(&self, k: &str) -> Symbol {
        // 現在のスコープから変数をサーチ
        match self.cur_scope {
            Scope::Global => {
                self.sym_table
                    .search(&self.cur_scope, k)
                    .expect("asm.rs(generate_var_symbol): error option value")
            }
            _ => {
                // もし、ローカルスコープで存在しない場合、Globalから検索
                self.sym_table.search(&self.cur_scope, k)
                    .unwrap_or_else(||
                        self.sym_table.search(&Scope::Global, k).expect("asm.rs(generate_var_symbol): error option value")
                )
            }
        }
   }

    // 左辺値変数アドレス取得
    fn generate_lvalue_address(&mut self, a: &AstType) {
        let (sym, name) = match *a {
            AstType::Variable(_, _, ref s) => (self.get_var_symbol(s), s),
            _ => panic!(format!(
                "asm.rs(generate_lvalue_address): Not Support AstType {:?}",
                a
            )),
        };

        // アドレスをraxレジスタへ転送
        self.inst = match sym.scope {
            Scope::Global => format!("{}{}", self.inst, self.gen_asm().lea_glb(name)),
            _ => match sym.strt {
                Structure::Array(_) => {
                    format!("{}{}", self.inst, self.gen_asm().lea(sym.size as i64))
                }
                _ => format!("{}{}", self.inst, self.gen_asm().lea(sym.offset as i64 + 8))
            }
        };
        self.inst = format!("{}{}", self.inst, self.gen_asm().push("rax"));
    }

    // 後置インクリメント
    fn generate_post_inc(&mut self, a: &AstType) {
        self.generate_lvalue_address(a);

        match *a {
            AstType::Variable(_, ref s, _) => match s {
                Structure::Identifier => {
                    self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rcx"));
                    self.inst =
                        format!("{}{}", self.inst, self.gen_asm().mov_src("rcx", "rax", 0));
                    self.inst = format!("{}{}", self.inst, self.gen_asm().push("rax"));
                    self.inst = format!("{}{}", self.inst, self.gen_asm().add_imm(1, "rax"));
                    self.inst =
                        format!("{}{}", self.inst, self.gen_asm().mov_dst("rax", "rcx", 0));
                }
                Structure::Pointer => {
                    self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rcx"));
                    self.inst =
                        format!("{}{}", self.inst, self.gen_asm().mov_src("rcx", "rax", 0));
                    self.inst = format!("{}{}", self.inst, self.gen_asm().push("rax"));
                    self.inst = format!("{}{}", self.inst, self.gen_asm().add_imm(8, "rax"));
                    self.inst =
                        format!("{}{}", self.inst, self.gen_asm().mov_dst("rax", "rcx", 0));
                }
                _ => panic!(format!("asm.rs(generate_post_inc): Not Support Structure {:?}", s)),
            },
            _ => panic!(format!("asm.rs(generate_post_inc): Not Support AstType {:?}",
                a
            )),
        }
    }

    // 後置デクリメント
    fn generate_post_dec(&mut self, a: &AstType) {
        self.generate_lvalue_address(a);

        match *a {
            AstType::Variable(_, ref s, _) => match s {
                Structure::Identifier => {
                    self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rcx"));
                    self.inst =
                        format!("{}{}", self.inst, self.gen_asm().mov_src("rcx", "rax", 0));
                    self.inst = format!("{}{}", self.inst, self.gen_asm().push("rax"));
                    self.inst = format!("{}{}", self.inst, self.gen_asm().sub_imm(1, "rax"));
                    self.inst =
                        format!("{}{}", self.inst, self.gen_asm().mov_dst("rax", "rcx", 0));
                }
                Structure::Pointer => {
                    self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rcx"));
                    self.inst =
                        format!("{}{}", self.inst, self.gen_asm().mov_src("rcx", "rax", 0));
                    self.inst = format!("{}{}", self.inst, self.gen_asm().push("rax"));
                    self.inst = format!("{}{}", self.inst, self.gen_asm().sub_imm(8, "rax"));
                    self.inst =
                        format!("{}{}", self.inst, self.gen_asm().mov_dst("rax", "rcx", 0));
                }
                _ => panic!(format!(
                    "asm.rs(generate_post_dec): Not Support Structure {:?}",
                    s
                )),
            },
            _ => panic!(format!(
                "asm.rs(generate_post_dec): Not Support AstType {:?}",
                a
            )),
        }
    }

    // 前置インクリメント
    fn generate_pre_inc(&mut self, a: &AstType) {
        self.generate_lvalue_address(a);

        match *a {
            AstType::Variable(_, ref s, _) => match s {
                Structure::Identifier => {
                    self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rcx"));
                    self.inst =
                        format!("{}{}", self.inst, self.gen_asm().mov_src("rcx", "rax", 0));
                    self.inst = format!("{}{}", self.inst, self.gen_asm().add_imm(1, "rax"));
                    self.inst =
                        format!("{}{}", self.inst, self.gen_asm().mov_dst("rax", "rcx", 0));
                    self.inst = format!("{}{}", self.inst, self.gen_asm().push("rax"));
                }
                Structure::Pointer => {
                    self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rcx"));
                    self.inst =
                        format!("{}{}", self.inst, self.gen_asm().mov_src("rcx", "rax", 0));
                    self.inst = format!("{}{}", self.inst, self.gen_asm().add_imm(8, "rax"));
                    self.inst =
                        format!("{}{}", self.inst, self.gen_asm().mov_dst("rax", "rcx", 0));
                    self.inst = format!("{}{}", self.inst, self.gen_asm().push("rax"));
                }
                _ => panic!(format!(
                    "asm.rs(generate_pre_inc): Not Support Structure {:?}",
                    s
                )),
            },
            _ => panic!(format!(
                "asm.rs(generate_pre_inc): Not Support AstType {:?}",
                a
            )),
        }
    }

    // 前置デクリメント
    fn generate_pre_dec(&mut self, a: &AstType) {
        self.generate_lvalue_address(a);

        match *a {
            AstType::Variable(_, ref s, _) => match s {
                Structure::Identifier => {
                    self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rcx"));
                    self.inst =
                        format!("{}{}", self.inst, self.gen_asm().mov_src("rcx", "rax", 0));
                    self.inst = format!("{}{}", self.inst, self.gen_asm().sub_imm(1, "rax"));
                    self.inst =
                        format!("{}{}", self.inst, self.gen_asm().mov_dst("rax", "rcx", 0));
                    self.inst = format!("{}{}", self.inst, self.gen_asm().push("rax"));
                }
                Structure::Pointer => {
                    self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rcx"));
                    self.inst =
                        format!("{}{}", self.inst, self.gen_asm().mov_src("rcx", "rax", 0));
                    self.inst = format!("{}{}", self.inst, self.gen_asm().sub_imm(8, "rax"));
                    self.inst =
                        format!("{}{}", self.inst, self.gen_asm().mov_dst("rax", "rcx", 0));
                    self.inst = format!("{}{}", self.inst, self.gen_asm().push("rax"));
                }
                _ => panic!(format!(
                    "asm.rs(generate_pre_dec): Not Support Structure {:?}",
                    s
                )),
            },
            _ => panic!(format!(
                "asm.rs(generate_pre_dec): Not Support AstType {:?}",
                a
            )),
        }
    }

    // ポインタ同士の加算
    fn generate_plus_with_pointer(&mut self, a: &AstType, b: &AstType) {
        self.generate(a);
        self.generate(b);
        self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rax"));
        self.inst = format!("{}{}", self.inst, self.gen_asm().mov_imm("rcx", 8));
        self.inst = format!("{}{}", self.inst, self.gen_asm().mul("rcx"));
        self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rcx"));
        self.inst = format!("{}{}", self.inst, self.gen_asm().add("rax", "rcx"));
        self.inst = format!("{}{}", self.inst, self.gen_asm().push("rcx"));
    }

    // variable同士の加算
    fn generate_plus_variable(&mut self, a: &AstType, b: &AstType, s: &Structure) {
        match s {
            Structure::Array(_) => self.generate_plus_with_pointer(a, b),
            _ => {
                self.generate(a);
                self.generate(b);

                // 加算処理
                self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rcx"));
                self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rax"));
                self.inst = format!("{}{}", self.inst, self.gen_asm().plus());
                self.inst = format!("{}{}", self.inst, self.gen_asm().push("rax"));
            }
        }
    }

    // 加算
    fn generate_plus(&mut self, a: &AstType, b: &AstType) {
        match (a, b) {
            // ポインタ演算チェック
            (AstType::Variable(ref _t1, ref s1, _), _) if *s1 == Structure::Pointer => {
                self.generate_plus_with_pointer(a, b)
            }
            (AstType::Variable(ref _t1, ref s1, _), _) => self.generate_plus_variable(a, b, s1),
            _ => {
                self.generate(a);
                self.generate(b);

                // 加算処理
                self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rcx"));
                self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rax"));
                self.inst = format!("{}{}", self.inst, self.gen_asm().plus());
                self.inst = format!("{}{}", self.inst, self.gen_asm().push("rax"));
            }
        }
    }

    // ポインタ同士の減算
    fn generate_minus_with_pointer(&mut self, a: &AstType, b: &AstType) {
        self.generate(a);
        self.generate(b);
        self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rax"));
        self.inst = format!("{}{}", self.inst, self.gen_asm().mov_imm("rcx", 8));
        self.inst = format!("{}{}", self.inst, self.gen_asm().mul("rcx"));
        self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rcx"));
        self.inst = format!("{}{}", self.inst, self.gen_asm().sub("rax", "rcx"));
        self.inst = format!("{}{}", self.inst, self.gen_asm().push("rcx"));
    }

    // 減算
    fn generate_minus(&mut self, a: &AstType, b: &AstType) {
        match (a, b) {
            (AstType::Variable(ref _t1, ref s1, _), AstType::Variable(ref t2, _, _))
                if *s1 == Structure::Pointer && (*t2 == Type::Int || *t2 == Type::Char) =>
            {
                self.generate_minus_with_pointer(a, b)
            }
            (AstType::Variable(ref _t1, ref s1, _), AstType::Factor(_))
                if *s1 == Structure::Pointer =>
            {
                self.generate_minus_with_pointer(a, b)
            }
            _ => {
                self.generate(a);
                self.generate(b);

                // 減算処理
                self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rcx"));
                self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rax"));
                self.inst = format!("{}{}", self.inst, self.gen_asm().minus());
                self.inst = format!("{}{}", self.inst, self.gen_asm().push("rax"));
            }
        }
    }

    // 演算子生成.
    fn generate_operator(&mut self, ast: &AstType, a: &AstType, b: &AstType) {
        self.generate(a);
        self.generate(b);

        // 各演算子評価.
        self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rcx"));
        self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rax"));
        self.inst = format!("{}{}", self.inst, self.operator(ast));

        // 演算子に応じて退避するレジスタを変更.
        match *ast {
            AstType::Remainder(_, _) => {
                self.inst = format!("{}{}", self.inst, self.gen_asm().push("rdx"));
            }
            _ => {
                self.inst = format!("{}{}", self.inst, self.gen_asm().push("rax"));
            }
        }
    }

    // アドレス演算子.
    fn generate_address(&mut self, a: &AstType) {
        match *a {
            AstType::Variable(ref _t, ref _s, ref a) => {
                let sym = self.get_var_symbol(a);
                let pos = sym.offset as i64 + 8;
                self.inst = format!("{}{}", self.inst, self.gen_asm().lea(pos));
                self.inst = format!("{}{}", self.inst, self.gen_asm().push("rax"));
            }
            _ => panic!("{} {}: Not Support Ast {:?}", file!(), line!(), a),
        }
    }

    // 間接演算子.
    fn generate_indirect(&mut self, a: &AstType) {
        self.generate(a);
        self.inst = format!("{}{}", self.inst, self.gen_asm().pop("rax"));
        self.inst = format!("{}{}", self.inst, self.gen_asm().mov_src("rax", "rcx", 0));
        self.inst = format!("{}{}", self.inst, self.gen_asm().push("rcx"));
    }

    // 演算子アセンブラ生成.
    fn operator(&self, ope: &AstType) -> String {
        match *ope {
            AstType::Multiple(_, _) => self.gen_asm().multiple(),
            AstType::Equal(_, _) => self.gen_asm().equal(),
            AstType::NotEqual(_, _) => self.gen_asm().not_equal(),
            AstType::LessThan(_, _) => self.gen_asm().less_than(),
            AstType::GreaterThan(_, _) => self.gen_asm().greater_than(),
            AstType::LessThanEqual(_, _) => self.gen_asm().less_than_equal(),
            AstType::GreaterThanEqual(_, _) => self.gen_asm().greater_than_equal(),
            AstType::LeftShift(_, _) => self.gen_asm().left_shift(),
            AstType::RightShift(_, _) => self.gen_asm().right_shift(),
            AstType::BitAnd(_, _) => self.gen_asm().bit_and(),
            AstType::BitOr(_, _) => self.gen_asm().bit_or(),
            AstType::BitXor(_, _) => self.gen_asm().bit_xor(),
            AstType::Division(_, _) | AstType::Remainder(_, _) => self.gen_asm().bit_division(),
            _ => process::abort(),
        }
    }

    // 文字列リテラル生成
    fn generate_string_literal(&mut self, a: &AstType) {
        match a {
            AstType::StringLiteral(s, i) => {
                self.const_literal = format!("{}  .text\n", self.const_literal);
                self.const_literal = format!("{}.LC{}:\n", self.const_literal, i);
                self.const_literal = format!("{}  .string \"{}\"\n", self.const_literal, s);
            }
            _ => panic!("asm.rs(generate_string_literal): not support {:?}", a),
        }
    }

    // 文字列リテラル命令
    fn generate_string(&mut self, _s: &str, i: usize) {
        self.inst = format!("{}  movq $.LC{}, %rax\n", self.inst, i);
        self.inst = format!("{}{}", self.inst, self.gen_asm().push("rax"));
    }

    // ラベル命令.
    fn generate_label_inst(&mut self, no: usize) {
        self.inst = format!("{}{}", self.inst, self.gen_asm().label(no));
    }

    // jmp命令生成.
    fn generate_jmp_inst(&mut self, no: usize) {
        self.inst = format!("{}{}", self.inst, self.gen_asm().jmp(no));
    }

    // je命令生成.
    fn generate_je_inst(&mut self, no: usize) {
        self.inst = format!("{}{}", self.inst, self.gen_asm().je(no));
    }

    // jne命令生成.
    fn generate_jne_inst(&mut self, no: usize) {
        self.inst = format!("{}{}", self.inst, self.gen_asm().jne(no));
    }

    // cmp命令生成.
    fn generate_cmp_inst(&mut self, f: usize, r: &str) {
        self.inst = format!("{}{}", self.inst, self.gen_asm().cmpl(f, r));
    }
}
