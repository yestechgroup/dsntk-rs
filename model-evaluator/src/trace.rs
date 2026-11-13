//! Evaluation trace: the record of which decision-table rules matched
//! during one evaluation.
//!
//! Decision provenance — the "why did the system make this decision"
//! audit trail — needs to know which rules of which decision tables
//! fired. The evaluator closures that decide this are built once and
//! evaluate many times, so the trace is collected through a thread-local
//! sink: [with_trace_sink] installs the sink for the duration of one
//! evaluation, and every decision-table evaluator that runs on the same
//! thread records the numbers of its matching rules.

use std::cell::RefCell;
use std::rc::Rc;

/// The trace of one evaluated decision table: the table's identifier and
/// the zero-based numbers of the rules that matched (in table order).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableTrace {
  /// Identifier of the evaluated decision table.
  pub table_id: String,
  /// Numbers of the rules that matched, in table order.
  pub matched_rules: Vec<usize>,
}

/// The sink collected traces are pushed into.
pub type TraceSink = Rc<RefCell<Vec<TableTrace>>>;

thread_local! {
  /// The trace sink active on the current thread, when any.
  static TRACE_SINK: RefCell<Option<TraceSink>> = const { RefCell::new(None) };
}

/// Runs the specified evaluation with a fresh trace sink installed, and
/// returns the evaluation's result together with the collected traces.
pub fn with_trace_sink<T>(evaluation: impl FnOnce() -> T) -> (T, Vec<TableTrace>) {
  let sink: TraceSink = Rc::new(RefCell::new(Vec::new()));
  TRACE_SINK.with(|cell| *cell.borrow_mut() = Some(Rc::clone(&sink)));
  let result = evaluation();
  TRACE_SINK.with(|cell| *cell.borrow_mut() = None);
  let traces = match Rc::try_unwrap(sink) {
    Ok(inner) => inner.into_inner(),
    Err(sink) => sink.borrow().clone(),
  };
  (result, traces)
}

/// Records the trace of one evaluated decision table into the active
/// sink, when one is installed.
pub(crate) fn record_table_trace(table_id: &str, matched_rules: Vec<usize>) {
  TRACE_SINK.with(|cell| {
    if let Some(sink) = cell.borrow().as_ref() {
      sink.borrow_mut().push(TableTrace {
        table_id: table_id.to_string(),
        matched_rules,
      });
    }
  });
}
