use super::*;

/**
 * Removes all unreachable statements from the program.
 * It only does so _within_ functions - it does not do cross-function optimizations.
 * It only cares about whether the expr.vartype is None, it does not look at struct_types or anything else.
 * It also propagates unreachability out of sequences etc.
 * The second return value is true if the program got changed, or false otherwise.
 */
pub fn optimize(mut program: Program) -> (Program, bool) {
    let mut changed = false;
    for func in &mut program.funcs {
        changed |= optimize_func(func);
    }
    (program, changed)
}

/**
 * Removes all unreachable statements from the function.
 * The return value is true if the function got changed, or false otherwise.
 */
fn optimize_func(func: &mut Func) -> bool {
    optimize_expr(&mut func.expr)
}

fn optimize_expr(expr: &mut Expr) -> bool {
    match &mut expr.kind {
        ExprKind::PrimFunc {
            funcidxs: _,
            closure,
        } => optimize_expr(&mut **closure),
        ExprKind::TypeCast {
            test,
            expected: _,
            create_narrow_local: _,
            true_expr,
            false_expr,
        } => {
            optimize_expr(&mut **test)
                | optimize_expr(&mut **true_expr)
                | optimize_expr(&mut **false_expr)
        }
        ExprKind::PrimAppl { prim_inst: _, args } => args
            .iter_mut()
            .fold(false, |prev, arg| prev | optimize_expr(arg)),
        ExprKind::Appl { func, args } => {
            optimize_expr(func)
                | args
                    .iter_mut()
                    .fold(false, |prev, arg| prev | optimize_expr(arg))
        }
        ExprKind::DirectAppl { funcidx: _, args } => args
            .iter_mut()
            .fold(false, |prev, arg| prev | optimize_expr(arg)),
        ExprKind::Conditional {
            cond,
            true_expr,
            false_expr,
        } => {
            optimize_expr(&mut **cond)
                | optimize_expr(&mut **true_expr)
                | optimize_expr(&mut **false_expr)
        }
        ExprKind::Declaration {
            local: _,
            expr: expr2,
        } => {
            let res = optimize_expr(&mut **expr2);
            expr.vartype = expr2.vartype;
            res
        }
        ExprKind::Assign { target: _, expr } => optimize_expr(&mut **expr),
        ExprKind::Return { expr } => optimize_expr(&mut **expr),
        ExprKind::Sequence { content } => {
            let tmp_content = std::mem::take(content);
            let mut changed = false;
            let mut new_content = Vec::new();
            let tmp_content_len = tmp_content.len();
            for mut expr2 in tmp_content {
                changed |= optimize_expr(&mut expr2);
                let is_none = expr2.vartype.is_none();
                new_content.push(expr2);
                if is_none {
                    break;
                }
            }
            changed |= new_content.len() != tmp_content_len;
            expr.vartype = new_content
                .last()
                .map_or_else(|| None, |expr2| expr2.vartype);
            *content = new_content;
            changed
        }
        _ => false,
    }
}
