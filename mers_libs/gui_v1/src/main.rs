use std::{
    io::{self, Read},
    sync::mpsc,
    time::Duration,
};

use iced::{
    executor, time,
    widget::{button, column, row, text},
    Application, Command, Element, Renderer, Settings, Subscription, Theme,
};
use mers::{
    libs::inlib::{MyLib, MyLibTask},
    script::{
        val_data::{VData, VDataEnum},
        val_type::{VSingleType, VType},
    },
};

/*

Path: Vec<usize>

*/

fn single_gui_element() -> VType {
    VType {
        types: vec![
            VSingleType::EnumVariantS("Row".to_string(), VSingleType::Tuple(vec![]).to()),
            VSingleType::EnumVariantS("Column".to_string(), VSingleType::Tuple(vec![]).to()),
            VSingleType::EnumVariantS("Text".to_string(), VSingleType::String.to()),
            VSingleType::EnumVariantS(
                "Button".to_string(),
                VType {
                    types: vec![VSingleType::Tuple(vec![]), VSingleType::String],
                },
            ),
        ],
    }
}
fn single_gui_update() -> VType {
    VType {
        types: vec![VSingleType::EnumVariantS(
            "ButtonPressed".to_string(),
            VSingleType::List(VSingleType::Int.to()).to(),
        )],
    }
}

fn main() {
    let (sender, recv) = mpsc::channel();
    let (sender2, recv2) = mpsc::channel();
    std::thread::spawn(move || {
        let recv = recv2;
        let (mut my_lib, mut run) = MyLib::new(
            "GUI-Iced".to_string(),
            (0, 0),
            "A basic GUI library for mers.".to_string(),
            vec![
                (
                    "gui_init".to_string(),
                    vec![],
                    VSingleType::List(VSingleType::Int.to()).to(),
                ),
                (
                    "gui_updates".to_string(),
                    vec![],
                    VSingleType::List(single_gui_update()).to(),
                ),
                (
                    "set_title".to_string(),
                    vec![VSingleType::String.to()],
                    VSingleType::Tuple(vec![]).to(),
                ),
                (
                    "gui_add".to_string(),
                    vec![
                        VSingleType::List(VSingleType::Int.to()).to(),
                        single_gui_element(),
                    ],
                    VSingleType::List(VSingleType::Int.to()).to(),
                ),
                (
                    "gui_remove".to_string(),
                    vec![VSingleType::List(VSingleType::Int.to()).to()],
                    VSingleType::Tuple(vec![]).to(),
                ),
            ],
        );
        let mut stdin = std::io::stdin().lock();
        let mut stdout = std::io::stdout().lock();
        let mut layout = Layout::Row(vec![]);
        loop {
            run = match my_lib.run(run, &mut stdin, &mut stdout) {
                MyLibTask::None(v) | MyLibTask::FinishedInit(v) => v,
                MyLibTask::RunFunction(mut f) => {
                    let return_value = match f.function {
                        0 => VDataEnum::List(VSingleType::Int.to(), vec![]).to(),
                        1 => {
                            let mut v = vec![];
                            while let Ok(recv) = recv.try_recv() {
                                match recv {
                                    MessageAdv::ButtonPressed(path) => v.push(
                                        VDataEnum::EnumVariant(
                                            my_lib.get_enum("ButtonPressed").unwrap(),
                                            Box::new(
                                                VDataEnum::List(VSingleType::Int.to(), path).to(),
                                            ),
                                        )
                                        .to(),
                                    ),
                                }
                            }
                            VDataEnum::List(single_gui_update(), v).to()
                        }
                        2 => {
                            // set_title
                            if let VDataEnum::String(new_title) = f.args.remove(0).data {
                                sender.send(Task::SetTitle(new_title)).unwrap();
                                VDataEnum::Tuple(vec![]).to()
                            } else {
                                unreachable!()
                            }
                        }
                        3 => {
                            // gui_add
                            if let (layout_data, VDataEnum::List(_, path)) =
                                (f.args.remove(1).data, f.args.remove(0).data)
                            {
                                let path: Vec<usize> = path
                                    .into_iter()
                                    .map(|v| {
                                        if let VDataEnum::Int(v) = v.data {
                                            v as _
                                        } else {
                                            unreachable!()
                                        }
                                    })
                                    .collect();
                                let lo = layout_from_vdata(&my_lib, layout_data);
                                let layout_inner = layout.get_mut(&path, 0);
                                let new_path: Vec<_> = path
                                    .iter()
                                    .map(|v| VDataEnum::Int(*v as _).to())
                                    .chain(
                                        [VDataEnum::Int(layout_inner.len() as _).to()].into_iter(),
                                    )
                                    .collect();
                                layout_inner.add(lo.clone());
                                sender.send(Task::LAdd(path, lo)).unwrap();
                                VDataEnum::List(VSingleType::Int.to(), new_path).to()
                            } else {
                                unreachable!()
                            }
                        }
                        4 => {
                            // gui_remove
                            if let VDataEnum::List(_, path) = f.args.remove(0).data {
                                let mut path: Vec<usize> = path
                                    .into_iter()
                                    .map(|v| {
                                        if let VDataEnum::Int(v) = v.data {
                                            v as _
                                        } else {
                                            unreachable!()
                                        }
                                    })
                                    .collect();
                                if let Some(remove_index) = path.pop() {
                                    let layout_inner = layout.get_mut(&path, 0);
                                    layout_inner.remove(remove_index);
                                    path.push(remove_index);
                                    sender.send(Task::LRemove(path)).unwrap();
                                }
                                VDataEnum::Tuple(vec![]).to()
                            } else {
                                unreachable!()
                            }
                        }
                        _ => unreachable!(),
                    };
                    f.done(&mut stdout, return_value)
                }
            }
        }
    });
    App::run(Settings::with_flags((recv, sender2))).unwrap();
}

fn layout_from_vdata(my_lib: &MyLib, d: VDataEnum) -> Layout {
    let row = my_lib.get_enum("Row").unwrap();
    let col = my_lib.get_enum("Column").unwrap();
    let text = my_lib.get_enum("Text").unwrap();
    let button = my_lib.get_enum("Button").unwrap();
    if let VDataEnum::EnumVariant(variant, inner_data) = d {
        if variant == row {
            Layout::Row(vec![])
        } else if variant == col {
            Layout::Column(vec![])
        } else if variant == text {
            Layout::Text(if let VDataEnum::String(s) = inner_data.data {
                s
            } else {
                String::new()
            })
        } else if variant == button {
            Layout::Button(Box::new(Layout::Text(
                if let VDataEnum::String(s) = inner_data.data {
                    s
                } else {
                    String::new()
                },
            )))
        } else {
            unreachable!()
        }
    } else {
        unreachable!()
    }
}

enum Task {
    SetTitle(String),
    LAdd(Vec<usize>, Layout),
    LSet(Vec<usize>, Layout),
    LRemove(Vec<usize>),
}

struct App {
    title: String,
    recv: mpsc::Receiver<Task>,
    sender: mpsc::Sender<MessageAdv>,
    buttons: Vec<Vec<usize>>,
    layout: Layout,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Tick,
    ButtonPressed(usize),
}
enum MessageAdv {
    ButtonPressed(Vec<VData>),
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = (mpsc::Receiver<Task>, mpsc::Sender<MessageAdv>);
    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                title: format!("mers gui (using iced)..."),
                recv: flags.0,
                sender: flags.1,
                buttons: vec![],
                layout: Layout::Column(vec![]),
            },
            Command::none(),
        )
    }
    fn title(&self) -> String {
        format!("{}", self.title)
    }
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        let mut commands = vec![];
        match message {
            Message::Tick => {
                let mut changed_layout = false;
                while let Ok(task) = self.recv.try_recv() {
                    match task {
                        Task::SetTitle(t) => {
                            self.title = t;
                        }
                        Task::LAdd(path, add) => {
                            changed_layout = true;
                            self.layout.get_mut(&path, 0).add(add);
                        }
                        Task::LSet(path, add) => {
                            changed_layout = true;
                            *self.layout.get_mut(&path, 0) = add;
                        }
                        Task::LRemove(mut path) => {
                            if let Some(last) = path.pop() {
                                changed_layout = true;
                                self.layout.get_mut(&path, 0).remove(last);
                            }
                        }
                    }
                }
                if changed_layout {
                    self.calc_layout_stats();
                }
            }
            Message::ButtonPressed(bid) => self
                .sender
                .send(MessageAdv::ButtonPressed(
                    self.buttons[bid]
                        .iter()
                        .map(|v| VDataEnum::Int(*v as _).to())
                        .collect(),
                ))
                .unwrap(),
        }
        Command::batch(commands)
    }
    fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_millis(10)).map(|_| Message::Tick)
    }
    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        self.viewl(&mut vec![], &self.layout, &mut 0)
    }
}
impl App {
    fn viewl(
        &self,
        p: &mut Vec<usize>,
        l: &Layout,
        current_button: &mut usize,
    ) -> Element<'_, <Self as Application>::Message, Renderer<<Self as Application>::Theme>> {
        match l {
            Layout::Row(v) => row(v
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    p.push(i);
                    let o = self.viewl(p, v, current_button);
                    p.pop();
                    o
                })
                .collect())
            .into(),
            Layout::Column(v) => column(
                v.iter()
                    .enumerate()
                    .map(|(i, v)| {
                        p.push(i);
                        let o = self.viewl(p, v, current_button);
                        p.pop();
                        o
                    })
                    .collect(),
            )
            .into(),
            Layout::Text(txt) => text(txt).into(),
            Layout::Button(content) => button({
                p.push(0);
                let o = self.viewl(p, content, current_button);
                p.pop();
                o
            })
            .on_press(Message::ButtonPressed({
                let o = *current_button;
                *current_button = *current_button + 1;
                o
            }))
            .into(),
        }
    }
    fn calc_layout_stats(&mut self) {
        self.buttons.clear();
        Self::calc_layout_stats_rec(&self.layout, &mut vec![], &mut self.buttons)
    }
    fn calc_layout_stats_rec(
        layout: &Layout,
        path: &mut Vec<usize>,
        buttons: &mut Vec<Vec<usize>>,
    ) {
        match layout {
            Layout::Row(v) | Layout::Column(v) => {
                for (i, v) in v.iter().enumerate() {
                    path.push(i);
                    Self::calc_layout_stats_rec(v, path, buttons);
                    path.pop();
                }
            }
            Layout::Button(c) => {
                buttons.push(path.clone());
                path.push(0);
                Self::calc_layout_stats_rec(c, path, buttons);
                path.pop();
            }
            Layout::Text(_) => (),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Layout {
    Row(Vec<Self>),
    Column(Vec<Self>),
    Text(String),
    Button(Box<Self>),
}
impl Layout {
    pub fn get_mut(&mut self, path: &Vec<usize>, index: usize) -> &mut Self {
        if index >= path.len() {
            return self;
        }
        match self {
            Self::Row(v) => v[path[index]].get_mut(path, index + 1),
            Self::Column(v) => v[path[index]].get_mut(path, index + 1),
            Self::Button(c) => c.as_mut().get_mut(path, index + 1),
            Self::Text(_) => {
                panic!("cannot index this layout type! ({:?})", self)
            }
        }
    }
    pub fn add(&mut self, add: Layout) {
        match self {
            Self::Row(v) | Self::Column(v) => v.push(add),
            _ => panic!("cannot add to this layout type! ({:?})", self),
        }
    }
    pub fn remove(&mut self, remove: usize) {
        match self {
            Self::Row(v) | Self::Column(v) => {
                if remove < v.len() {
                    v.remove(remove);
                }
            }
            _ => panic!("cannot add to this layout type! ({:?})", self),
        }
    }
    pub fn len(&self) -> usize {
        match self {
            Self::Row(v) | Self::Column(v) => v.len(),
            _ => panic!("cannot get len of this layout type! ({:?})", self),
        }
    }
}

trait DirectRead {
    fn nbyte(&mut self) -> Result<u8, io::Error>;
    fn nchar(&mut self) -> Result<char, io::Error>;
}
impl<T> DirectRead for T
where
    T: Read,
{
    fn nbyte(&mut self) -> Result<u8, io::Error> {
        let mut b = [0];
        self.read(&mut b)?;
        Ok(b[0])
    }
    fn nchar(&mut self) -> Result<char, io::Error> {
        Ok(self.nbyte()?.into())
    }
}
