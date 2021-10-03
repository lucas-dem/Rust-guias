use std::sync::mpsc::channel;
use std::sync::mpsc;
use std::thread;
use std::sync::Arc;
use std::sync::Mutex;

pub struct ThreadPool{
    threads: Vec<ThreadNumerado>,
    sender: mpsc::Sender<Mensaje>,
}

trait FnBox{
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

type Trabajo = Box<FnBox + Send + 'static>;

pub struct ThreadNumerado{
    id: usize,
    thread_numerado: Option<thread::JoinHandle<()>>,
}

impl ThreadNumerado{
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Mensaje>>>) -> ThreadNumerado {
        let thread_numerado = thread::spawn(move|| {
            loop{
                let mensaje = receiver.lock().unwrap().recv().unwrap(); 
                match mensaje{
                    Mensaje::NewJob(trabajo)=>{
                        println!("Thread {} tiene un trabajo, ejecutando",id);
                        trabajo.call_box();
                    },
                    Mensaje::Terminate => {
                        println!("Thread {} tiene que morir", id);
                        break;
                    },
                }
            }                                                           
        });

        ThreadNumerado {
            id,
            thread_numerado: Some(thread_numerado),
        }
    }
}

enum Mensaje {
    NewJob(Trabajo),
    Terminate,
}

impl Drop for ThreadPool{
    fn drop(&mut self){
        println!("Execute order 66: Enviando el mensaje que mata a todos los threads numerados ");
        for _ in &mut self.threads{
            self.sender.send(Mensaje::Terminate).unwrap();
        }

        println!("Matar al ultimo jedi: dando de baja a los threads numerados");

        for thread_numerado in &mut self.threads{
            println!("Matando al jedi {} : dando de baja al thread numero {}", thread_numerado.id, thread_numerado.id);

            if let Some(thread_numerado) = thread_numerado.thread_numerado.take(){
                thread_numerado.join().unwrap();
            }
        }
    }
}


impl ThreadPool{
    pub fn new(size: usize) -> ThreadPool{
        assert!(size>0);

        let (sender,receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver)); 

        let mut threads = Vec::with_capacity(size);

        for id in 0..size {
            threads.push(ThreadNumerado::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            threads,
            sender,
        }
    }

    pub fn ejecutar<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static
    {
        let trabajo = Box::new(f);

        self.sender.send(Mensaje::NewJob(trabajo)).unwrap();
    }
}

pub fn saludar_concurrente(){
    println!("Ojala ande, boquita el más grande");
}


fn main(){
    let pool = ThreadPool::new(4);
    pool.ejecutar(||{
        saludar_concurrente();
    });
}