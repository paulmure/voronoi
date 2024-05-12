use fortunes::{
    fortunes_algorithm,
    geometry::{BoundingBox, Point, Segment},
};
use leptos::*;
use ordered_float::OrderedFloat;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, MouseEvent};

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;
const POINT_RADIUS: f64 = 1.0;

fn clear_canvas(context: &CanvasRenderingContext2d) {
    context.set_fill_style(&JsValue::from_str("white"));
    context.fill_rect(0.0, 0.0, WIDTH as f64, HEIGHT as f64);
}

fn get_context(node_ref: NodeRef<html::Canvas>) -> CanvasRenderingContext2d {
    let canvas: HtmlElement<html::Canvas> = node_ref.get_untracked().unwrap();
    canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap()
}

fn draw_sites(context: &CanvasRenderingContext2d, sites: &Vec<Point>) {
    clear_canvas(context);
    for site in sites {
        context.set_stroke_style(&JsValue::from_str("red"));
        context.begin_path();
        context
            .arc(
                site.x.into(),
                site.y.into(),
                POINT_RADIUS,
                0.0,
                std::f64::consts::PI * 2.0,
            )
            .unwrap();
        context.stroke();
    }
}

fn draw_solution(context: &CanvasRenderingContext2d, sites: &Vec<Point>, edges: &Vec<Segment>) {
    draw_sites(context, sites);
    for [a, b] in edges {
        context.set_stroke_style(&JsValue::from_str("black"));
        context.begin_path();
        context.move_to(a.x.into(), a.y.into());
        context.line_to(b.x.into(), b.y.into());
        context.stroke();
    }

    // for (center, radius) in circles {
    //     context.set_stroke_style(&JsValue::from_str("red"));
    //     context.begin_path();
    //     context
    //         .arc(
    //             center.x.into(),
    //             center.y.into(),
    //             (*radius).into(),
    //             0.0,
    //             std::f64::consts::PI * 2.0,
    //         )
    //         .unwrap();
    //     context.stroke();
    // }
}

#[component]
fn App() -> impl IntoView {
    let bounding_box = BoundingBox::new(0.0.into(), WIDTH.into(), 0.0.into(), HEIGHT.into());

    let canvas_ref: NodeRef<html::Canvas> = create_node_ref::<html::Canvas>();
    create_effect(move |_| {
        clear_canvas(&get_context(canvas_ref));
    });

    let (sites, set_sites) = create_signal::<Vec<Point>>(vec![]);

    create_effect(move |_| {
        let context = &get_context(canvas_ref);
        draw_sites(context, &sites())
    });

    view! {
        <div class="my-10 flex w-screen flex-col items-center justify-center gap-5 text-gray-50">
            <h1 class="text-5xl">Voronoi Diagram</h1>
            <canvas
                width=WIDTH
                height=HEIGHT
                node_ref=canvas_ref
                on:click=move |e: MouseEvent| {
                    let x = e.offset_x();
                    let y = e.offset_y();
                    let site = Point {
                        x: OrderedFloat(x as f64),
                        y: OrderedFloat(y as f64),
                    };
                    set_sites.update(|ss: &mut Vec<Point>| ss.push(site));
                }
            >
            </canvas>
            <div class="flex flex-row items-center justify-center gap-10 text-2xl">
                <button
                    class="rounded bg-sky-300 px-4 py-2 font-bold text-slate-900 hover:bg-sky-500"
                    on:click=move |_| {
                        let edges = fortunes_algorithm(&sites(), &bounding_box);
                        draw_solution(&get_context(canvas_ref), &sites(), &edges);
                    }
                >

                    Solve
                </button>
                <button
                    class="rounded bg-sky-300 px-4 py-2 font-bold text-slate-900 hover:bg-sky-500"
                    on:click=move |_| set_sites.set(vec![])
                >
                    Clear
                </button>
            </div>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App)
}
