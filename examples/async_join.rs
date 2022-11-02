use circuits::{tokio, JoinTasks, Proc};

fn main() {
    // Join as JoinTask
    let mut tasks_a = (0..10).fold(JoinTasks::new(), |tasks, val| tasks.and(double_it(val)));
    let doubles_a = tasks_a.join().expect("Could not join A");

    // Join as future onto tokio Proc
    let doubles_b = tokio(async move {
        (0..10)
            .fold(JoinTasks::new(), |tasks, val| tasks.and(double_it(val)))
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(anyhow::Error::new)
    })
    .join()
    .expect("Could not join B");

    assert_eq!(doubles_a, doubles_b);
    assert_eq!(doubles_a, vec![0, 2, 4, 6, 8, 10, 12, 14, 16, 18]);
}

async fn double_it(val: usize) -> usize {
    val * 2
}
