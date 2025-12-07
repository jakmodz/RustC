use crate::tacky::{BinaryOp, TackyInstruction, UnaryOp, Val};

pub struct InstructionBuilder<'a> {
    instructions: &'a mut Vec<TackyInstruction>,
}

impl<'a> InstructionBuilder<'a> {
    pub fn new(instructions: &'a mut Vec<TackyInstruction>) -> Self {
        Self { instructions }
    }

    pub fn copy(&mut self, src: Val, dst: Val) -> &mut Self {
        self.instructions.push(TackyInstruction::Copy { src, dst });
        self
    }

    pub fn unary(&mut self, unary_op: UnaryOp, src: Val, dst: Val) -> &mut Self {
        self.instructions
            .push(TackyInstruction::Unary { unary_op, src, dst });
        self
    }

    pub fn binary(&mut self, binary_op: BinaryOp, src1: Val, src2: Val, dst: Val) -> &mut Self {
        self.instructions.push(TackyInstruction::Binary {
            binary_op,
            src1,
            src2,
            dst,
        });
        self
    }

    pub fn jump(&mut self, target: String) -> &mut Self {
        self.instructions.push(TackyInstruction::Jump { target });
        self
    }

    pub fn jump_if_zero(&mut self, cond: Val, target: String) -> &mut Self {
        self.instructions
            .push(TackyInstruction::JumpIfZero { cond, target });
        self
    }

    pub fn jump_if_not_zero(&mut self, cond: Val, target: String) -> &mut Self {
        self.instructions
            .push(TackyInstruction::JumpIfNotZero { cond, target });
        self
    }

    pub fn label(&mut self, identifier: String) -> &mut Self {
        self.instructions.push(TackyInstruction::Label(identifier));
        self
    }

    pub fn return_val(&mut self, val: Val) -> &mut Self {
        self.instructions.push(TackyInstruction::Return(val));
        self
    }
    pub fn fn_call(&mut self,name:String,params:Vec<Val>,dst:Val)->&mut Self{
        self.instructions.push(TackyInstruction::FnCall { 
            fn_name: name, 
            args: params, 
            dst 
        });
        self
    }
}
