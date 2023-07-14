use chrono::prelude::*;
use chrono::Duration;
use futures::StreamExt;
use material_yew::{list::ListIndex, MatButton, MatIconButton, MatList, MatListItem, MatTextField};
use yew::platform::spawn_local;
use yew::platform::time::interval;
use yew::prelude::*;

#[derive(PartialEq, Clone)]
enum TaskTimer {
    Running(DateTime<Utc>, Duration),
    Stopped(Duration),
}
impl TaskTimer {
    fn stop(&self, current_time: &DateTime<Utc>) -> Self {
        match self {
            Self::Running(instant, duration) => Self::Stopped(
                current_time.signed_duration_since(instant.to_owned()) + duration.to_owned(),
            ),
            Self::Stopped(_) => self.to_owned(),
        }
    }
    fn start(&self) -> Self {
        match self {
            TaskTimer::Running(_, _) => self.to_owned(),
            TaskTimer::Stopped(duration) => Self::Running(Utc::now(), duration.to_owned()),
        }
    }
    fn at(&self, current_time: &DateTime<Utc>) -> String {
        let duration = match self {
            Self::Running(instant, duration) => {
                current_time.signed_duration_since(instant.to_owned()) + duration.to_owned()
            }
            Self::Stopped(previously) => previously.to_owned(),
        };
        let s = duration.num_seconds();
        let m = s / 60;
        let h = m / 60;
        format!("{:02}:{:02}:{:02}", h, m % 60, s % 60)
    }
}

impl Default for TaskTimer {
    fn default() -> Self {
        Self::Stopped(Duration::nanoseconds(0))
    }
}

#[derive(PartialEq, Clone, Properties)]
struct AddTaskProps {
    on_add: Callback<String, ()>,
}

#[function_component(AddTask)]
fn add_task(AddTaskProps { on_add }: &AddTaskProps) -> Html {
    let on_add = on_add.clone();
    let text = use_state_eq(|| "".to_owned());
    let oninput = {
        let text = text.clone();
        Callback::from(move |value| text.set(value))
    };
    html! {
        <div>
            <MatTextField {oninput} label="Name" value={(*text).clone()}/>
            <span onclick={ move |_arg| {on_add.emit((*text).clone())}} >
                <MatIconButton icon="add" />
            </span>
        </div>
    }
}
#[derive(PartialEq, Clone, Properties)]
struct TaskProps {
    name: String,
    timer: TaskTimer,
    current_time: DateTime<Utc>,
    on_delete: Callback<()>,
}

#[function_component(Task)]
fn task(
    TaskProps {
        name,
        timer,
        current_time,
        on_delete,
    }: &TaskProps,
) -> Html {
    let on_delete = on_delete.clone();
    html! {
    <>
        <span onclick={move |_arg| {on_delete.emit(())}}>
            <MatButton label="delete" icon="delete" raised=true/>
        </span>
        <b>{format!("{} {}", name, timer.at(current_time))}</b>
    </>
    }
}

#[derive(PartialEq, Clone, Properties)]
struct TaskListProps {
    tasks: Vec<(String, TaskTimer)>,
    select_callback: Callback<ListIndex>,
    current_time: DateTime<Utc>,
    on_delete: Callback<usize>,
}

#[function_component(TaskList)]
fn task_list(
    TaskListProps {
        tasks,
        select_callback,
        current_time,
        on_delete,
    }: &TaskListProps,
) -> Html {
    html! {
        <MatList onaction={select_callback}>
            { tasks.iter().enumerate().map(|(index, (name, timer))| {
                let on_delete = {
                    let on_delete = on_delete.clone();
                    Callback::from(move |_arg| {on_delete.emit(index)})
                };
                html! {
                    <MatListItem>
                    <Task name={name.clone()} timer={timer.clone()} current_time={current_time.clone()} on_delete={on_delete}/>
                    </MatListItem>}
            }).collect::<Html>()
            }
        </MatList>
    }
}

#[derive(PartialEq, Clone, Properties)]
struct TimerProps {
    callback: Callback<()>,
}

#[function_component(Timer)]
fn timer(props: &TimerProps) -> Html {
    props.callback.emit(());
    html! {}
}
async fn update_current_time(handle: UseStateHandle<DateTime<Utc>>) {
    let date = js_sys::Date::new_0().to_iso_string().as_string().unwrap();
    let now = DateTime::parse_from_rfc3339(date.as_str()).unwrap();
    handle.set(now.into());
}

#[function_component(App)]
fn app() -> Html {
    let current_time = use_state_eq(Utc::now);
    let selected: UseStateHandle<Option<usize>> = use_state_eq(|| None);
    let list = use_state(|| -> Vec<(String, TaskTimer)> { vec![] });
    let on_select = {
        let selected = selected.clone();
        let list = list.clone();
        let current_time = current_time.clone();
        Callback::from(move |index| {
            if let ListIndex::Single(i) = index {
                let mut l = (*list).clone();

                if let Some(current) = *selected {
                    let (name, timer) = l.get(current).unwrap();
                    l[current] = (name.to_owned(), timer.stop(&current_time));
                }
                if let Some(new) = i {
                    let (name, timer) = l.get(new).unwrap();
                    l[new] = (name.to_owned(), timer.start());
                }
                list.set(l);
                selected.set(i);
            }
        })
    };
    let on_delete = {
        let list = list.clone();
        let selected = selected.clone();
        Callback::from(move |index| {
            let mut l = (*list).clone();
            l.remove(index);
            list.set(l);
            if let Some(sel) = *selected {
                if sel == index {
                    selected.set(None);
                }
            }
        })
    };
    let on_stop = {
        let on_select = on_select.clone();
        Callback::from(move |_| on_select.emit(ListIndex::Single(None)))
    };
    let add_task = {
        let list = list.clone();
        Callback::from(move |name: String| {
            let mut l = (*list).clone();
            l.push((name, Default::default()));
            list.set(l);
        })
    };
    let run_once = {
        let current_time = current_time.clone();
        use_callback(
            move |_: (), _| {
                let current_time = current_time.clone();
                spawn_local(async move {
                    interval(std::time::Duration::from_millis(500))
                        .map(|_| current_time.clone())
                        .for_each(update_current_time)
                        .await;
                });
            },
            (),
        )
    };
    let selected_task = (*selected)
        .and_then(|index| (*list).get(index))
        .map_or("", |(name, _)| name);

    html! {
        <>
            <h1>{ "Time Tracker" }</h1>
            <b>{format!("Selected task: {}", selected_task)}</b>
            <AddTask on_add={add_task}/>
            <TaskList tasks={(*list).clone()} select_callback={on_select} current_time={*current_time} on_delete={on_delete}/>
            <Timer callback={run_once}/>
            //{run_once.emit(())}
            if (*selected).is_some() {
                <span onclick={on_stop}>
                    <MatButton label="Stop" icon="stop" raised=true/>
                </span>
            }
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
