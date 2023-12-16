use core::ops::Add;

pub trait PipelineElement<Input, Output> {
    fn execute(&mut self, input: Input) -> Output;
}

// impl<Input, Output> PipelineElement<Input, Output> for dyn Fn(Input) -> Output {
//     fn execute(&mut self, input: Input) -> Output {
//         self(input)
//     }
// }

struct ClosurePipeline<Input, Output> {
    closure: fn(Input) -> Output,
}
impl<Input, Output> ClosurePipeline<Input, Output> {
    pub fn new(closure: fn(Input) -> Output) -> Self {
        Self { closure }
    }
}
impl<Input, Output> PipelineElement<Input, Output> for ClosurePipeline<Input, Output> {
    fn execute(&mut self, input: Input) -> Output {
        (self.closure)(input)
    }
}


fn execute_a_pipeline_i_guess<Input, Intermediate, Output>(
    input: Input,
    mut x: impl PipelineElement<Input, Intermediate>,
    mut y: impl PipelineElement<Intermediate, Output>,
) -> Output {
    y.execute(x.execute(input))
}

// struct AddOne;
// impl PipelineElement<isize, isize> for AddOne {
//     fn execute(&mut self, input: isize) -> isize {
//         input + 1
//     }
// }

fn add_one(x: isize) -> isize {
    x + 1
}

struct ConvertToFloat;
impl PipelineElement<isize, f32> for ConvertToFloat {
    fn execute(&mut self, input: isize) -> f32 {
        input as f32
    }
}

struct DivideByPi;
impl PipelineElement<f32, f32> for DivideByPi {
    fn execute(&mut self, input: f32) -> f32 {
        input / 3.14
    }
}

struct Sum<T: Add<Output = T> + Copy> {
    pub sum: T
}
impl<T: Add<Output = T> + Copy> PipelineElement<T, T> for Sum<T> {
    fn execute(&mut self, input: T) -> T {
        self.sum = self.sum + input;

        input
    }
}

// macro_rules! pipeline {
//     ($x:expr => $y:expr) => {
//         |input| { $y.execute($x.execute(input)) }
//     };
//     ($x:expr => $y:expr => $z:expr) => {
//         |input| { $z.execute($y.execute($x.execute(input))) }
//     };
// }

macro_rules! pipeline {
    ($x:expr) => {
        $x.execute(0)
    };

    ($x:expr => $($y:expr) => +) => {
        $x.execute(pipeline!($($y)=>*))
    }
}

pub fn test_pipeline() -> f32 {
    let mut x = ClosurePipeline::new(|x: isize| x+1);
    let mut y = ConvertToFloat;
    let mut z = DivideByPi;

    let result = |input| z.execute(y.execute(x.execute(input)));

    //let result = y.execute(y.execute(x.execute(0)));

    // pipeline!(x -> y -> z); ===> 
    /* 
    let pipeline!{
        controller inputs => demixer => (roll pid controller), (pitch pid controller) => remixer => servo outputs
    }
     */

    //let mut pipeline = pipeline!(x => ConvertToFloat => DivideByPi);

    // THIS IS THE WRONG ORDER. MODIFY THE MACRO SO THAT IT READS FROM LEFT TO RIGHT like on the line above
    let mut pipeline = pipeline!(DivideByPi => ConvertToFloat => x);

    //return pipeline(10);
    0.0
}
