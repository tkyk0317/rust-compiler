use std::process;
use ast::Expr;

/**
 * アセンブラ生成部.
 */
pub struct Asm {
    inst: String,
}

impl Asm {
    // コンストラクタ.
    pub fn new() -> Asm {
        // スタート部分設定.
        let mut start = format!(".global main\n");
        start = format!("{}{}", start, "main:\n");
        start = format!("{}{}", start, "  push %rbp\n");
        start = format!("{}{}", start, "  mov %rsp, %rbp\n");

        Asm { inst: start }
    }

    // アセンブラ取得
    pub fn get_inst(&self) -> String {
        // 終了部分を結合し、返却.
        let mut end = format!("  movl 0(%rsp), %eax\n");
        end = format!("{}{}", end, "  add $4, %rsp\n");
        end = format!("{}{}", end, "  pop %rbp\n");
        end = format!("{}{}", end, "  ret\n");
        format!("{}{}", self.inst, end)
    }

    // アセンブラ生成.
    pub fn generate(&mut self, ast: &Expr) {
        match *ast {
            Expr::Plus(ref a, ref b) |
            Expr::Minus(ref a, ref b) |
            Expr::Multiple(ref a, ref b) |
            Expr::Division(ref a, ref b) |
            Expr::Remainder(ref a, ref b) |
            Expr::Equal(ref a, ref b) |
            Expr::NotEqual(ref a, ref b) |
            Expr::LessThan(ref a, ref b) |
            Expr::GreaterThan(ref a, ref b)  => {
                self.generate(a);
                self.generate(b);

                // 各演算子評価.
                self.inst = format!("{}{}{}", self.inst, self.pop_stack("ecx"), self.pop_stack("eax"));
                self.inst = format!("{}{}", self.inst, self.operator(ast));

                // 演算子に応じて退避するレジスタを変更.
                match *ast {
                    Expr::Remainder(_, _) => self.inst = format!("{}{}", self.inst, self.push_stack("edx")),
                    _ => self.inst = format!("{}{}", self.inst, self.push_stack("eax"))
                }
            }
            Expr::Factor(a) => {
                // 数値.
                self.inst = format!("{}{}", self.inst, "  sub $4, %rsp\n");
                self.inst = format!("{}  movl ${}, 0(%rsp)\n", self.inst, a);
            }
        }
    }

    // スタックポップ.
    fn pop_stack(&self, reg: &str) -> String {
        format!("  movl 0(%rsp), %{}\n  add $4, %rsp\n", reg)
    }

    // プッシュスタック
    fn push_stack(&self, reg: &str) -> String {
        format!("  sub $4, %rsp\n  movl %{}, 0(%rsp)\n", reg)
    }

    // 演算子アセンブラ生成.
    fn operator(&self, ope: &Expr) -> String {
        match *ope {
            Expr::Multiple(_, _) => "  imull %ecx\n".to_string(),
            Expr::Plus(_, _) => "  addl %ecx, %eax\n".to_string(),
            Expr::Minus(_, _) => "  subl %ecx, %eax\n".to_string(),
            Expr::Division(_, _) | Expr::Remainder(_, _)  => "  movl $0, %edx\n  idivl %ecx\n".to_string(),
            Expr::Equal(_, _) => "  cmpl %ecx, %eax\n  sete %al\n  movzbl %al, %eax\n".to_string(),
            Expr::NotEqual(_, _) => "  cmpl %ecx, %eax\n  setne %al\n  movzbl %al, %eax\n".to_string(),
            Expr::LessThan(_, _) => "  cmpl %ecx, %eax\n  setl %al\n  movzbl %al, %eax\n".to_string(),
            Expr::GreaterThan(_, _) => "  cmpl %ecx, %eax\n  setg %al\n  movzbl %al, %eax\n".to_string(),
            _ => process::abort()
        }
    }
}
