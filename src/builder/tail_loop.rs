use crate::ops::{self, OpType};

use crate::hugr::{view::HugrView, NodeType};
use crate::types::{AbstractSignature, ClassicRow, SimpleRow};
use crate::{Hugr, Node};

use super::build_traits::SubContainer;
use super::handle::BuildHandle;
use super::{
    dataflow::{DFGBuilder, DFGWrapper},
    BuildError, Container, Dataflow, TailLoopID, Wire,
};

/// Builder for a [`ops::TailLoop`] node.
pub type TailLoopBuilder<B> = DFGWrapper<B, BuildHandle<TailLoopID>>;

impl<B: AsMut<Hugr> + AsRef<Hugr>> TailLoopBuilder<B> {
    pub(super) fn create_with_io(
        base: B,
        loop_node: Node,
        tail_loop: &ops::TailLoop,
    ) -> Result<Self, BuildError> {
        let signature =
            AbstractSignature::new_df(tail_loop.body_input_row(), tail_loop.body_output_row());
        let dfg_build = DFGBuilder::create_with_io(base, loop_node, signature, None)?;

        Ok(TailLoopBuilder::from_dfg_builder(dfg_build))
    }
    /// Set the outputs of the [`ops::TailLoop`], with `out_variant` as the value of the
    /// termination predicate, and `rest` being the remaining outputs
    pub fn set_outputs(
        &mut self,
        out_variant: Wire,
        rest: impl IntoIterator<Item = Wire>,
    ) -> Result<(), BuildError> {
        Dataflow::set_outputs(self, [out_variant].into_iter().chain(rest.into_iter()))
    }

    /// Get a reference to the [`ops::TailLoop`]
    /// that defines the signature of the [`ops::TailLoop`]
    pub fn loop_signature(&self) -> Result<&ops::TailLoop, BuildError> {
        if let OpType::TailLoop(tail_loop) = self.hugr().get_optype(self.container_node()) {
            Ok(tail_loop)
        } else {
            Err(BuildError::UnexpectedType {
                node: self.container_node(),
                op_desc: "crate::ops::TailLoop",
            })
        }
    }

    /// The output types of the child graph, including the predicate as the first.
    pub fn internal_output_row(&self) -> Result<SimpleRow, BuildError> {
        self.loop_signature().map(ops::TailLoop::body_output_row)
    }
}

impl<H: AsMut<Hugr> + AsRef<Hugr>> TailLoopBuilder<H> {
    /// Set outputs and finish, see [`TailLoopBuilder::set_outputs`]
    pub fn finish_with_outputs(
        mut self,
        out_variant: Wire,
        rest: impl IntoIterator<Item = Wire>,
    ) -> Result<<Self as SubContainer>::ContainerHandle, BuildError>
    where
        Self: Sized,
    {
        self.set_outputs(out_variant, rest)?;
        self.finish_sub_container()
    }
}

impl TailLoopBuilder<Hugr> {
    /// Initialize new builder for a [`ops::TailLoop`] rooted HUGR
    pub fn new(
        just_inputs: impl Into<ClassicRow>,
        inputs_outputs: impl Into<SimpleRow>,
        just_outputs: impl Into<ClassicRow>,
    ) -> Result<Self, BuildError> {
        let tail_loop = ops::TailLoop {
            just_inputs: just_inputs.into(),
            just_outputs: just_outputs.into(),
            rest: inputs_outputs.into(),
        };
        // TODO: Allow input resources to be specified
        let base = Hugr::new(NodeType::pure(tail_loop.clone()));
        let root = base.root();
        Self::create_with_io(base, root, &tail_loop)
    }
}

#[cfg(test)]
mod test {
    use cool_asserts::assert_matches;

    use crate::{
        builder::{
            test::{BIT, NAT},
            DataflowSubContainer, HugrBuilder, ModuleBuilder,
        },
        classic_row,
        hugr::ValidationError,
        ops::Const,
        type_row,
        types::ClassicType,
        Hugr,
    };

    use super::*;
    #[test]
    fn basic_loop() -> Result<(), BuildError> {
        let build_result: Result<Hugr, ValidationError> = {
            let mut loop_b = TailLoopBuilder::new(vec![], vec![BIT], vec![ClassicType::i64()])?;
            let [i1] = loop_b.input_wires_arr();
            let const_wire = loop_b.add_load_const(Const::i64(1)?)?;

            let break_wire = loop_b.make_break(loop_b.loop_signature()?.clone(), [const_wire])?;
            loop_b.set_outputs(break_wire, [i1])?;
            loop_b.finish_hugr()
        };

        assert_matches!(build_result, Ok(_));
        Ok(())
    }

    #[test]
    fn loop_with_conditional() -> Result<(), BuildError> {
        let build_result = {
            let mut module_builder = ModuleBuilder::new();
            let mut fbuild = module_builder.define_function(
                "main",
                AbstractSignature::new_df(type_row![BIT], type_row![NAT]).pure(),
            )?;
            let _fdef = {
                let [b1] = fbuild.input_wires_arr();
                let loop_id = {
                    let mut loop_b = fbuild.tail_loop_builder(
                        vec![(ClassicType::bit(), b1)],
                        vec![],
                        classic_row![ClassicType::i64()],
                    )?;
                    let signature = loop_b.loop_signature()?.clone();
                    let const_wire = loop_b.add_load_const(Const::true_val())?;
                    let [b1] = loop_b.input_wires_arr();
                    let conditional_id = {
                        let predicate_inputs = vec![type_row![]; 2];
                        let output_row = loop_b.internal_output_row()?;
                        let mut conditional_b = loop_b.conditional_builder(
                            (predicate_inputs, const_wire),
                            vec![(BIT, b1)],
                            output_row,
                        )?;

                        let mut branch_0 = conditional_b.case_builder(0)?;
                        let [b1] = branch_0.input_wires_arr();

                        let continue_wire = branch_0.make_continue(signature.clone(), [b1])?;
                        branch_0.finish_with_outputs([continue_wire])?;

                        let mut branch_1 = conditional_b.case_builder(1)?;
                        let [_b1] = branch_1.input_wires_arr();

                        let wire = branch_1.add_load_const(Const::i64(2)?)?;
                        let break_wire = branch_1.make_break(signature, [wire])?;
                        branch_1.finish_with_outputs([break_wire])?;

                        conditional_b.finish_sub_container()?
                    };

                    loop_b.finish_with_outputs(conditional_id.out_wire(0), [])?
                };

                fbuild.finish_with_outputs(loop_id.outputs())?
            };
            module_builder.finish_hugr()
        };

        assert_matches!(build_result, Ok(_));

        Ok(())
    }
}
