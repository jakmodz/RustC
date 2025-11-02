

#[derive(Debug)]
pub struct AsmProgram{
    pub function: AsmFunction,
}
#[derive(Debug)]
pub struct AsmFunction{
    pub name: String,
    pub instructions: Vec<Instruction>
}

#[derive(Debug)]
pub enum Instruction{
    Mov{src: Operand,dst:Operand},
    Ret
}

#[derive(Debug)]
pub enum Operand{
    Imn(i64),
    Register
}
