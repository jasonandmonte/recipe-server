// Adapted from code by Bart Massey

mod recipe;

use std::collections::HashSet;
use leptos::prelude::*;

#[component]
pub fn EnterInput(set_endpoint: WriteSignal<String>) -> impl IntoView {
    // Create a signal to store the current input value
    let (input_text, set_input_text) = signal("".to_string());

    // Define the action to be performed on Enter
    let handle_enter_action = move |_| {
        // This closure needs to capture 'input_text' and 'set_submitted_text'
        // to read the current input and update the submitted text.
        let current_input = input_text.get(); // Get the current value from the signal
        if !current_input.trim().is_empty() {
            set_endpoint.set(format!("recipe/{}", current_input));
        }
    };

    view! {
        <div>
            "Find a recipe: " <input
                type="text"
                // Bind the input's value to the signal
                prop:value=input_text
                // Update the signal when the input changes
                on:input=move |ev| {
                    set_input_text.set(event_target_value(&ev));
                }
                // Listen for keydown events
                on:keydown=move |ev: web_sys::KeyboardEvent| {
                    if ev.key() == "Enter" {
                        handle_enter_action(ev);
                    }
                }
                placeholder="Recipe ID"
            />
        </div>
    }
}

fn format_tags(tags: &HashSet<String>) -> String {
    let taglist: Vec<&str> = tags.iter().map(String::as_ref).collect();
    taglist.join(", ")
}

fn fetch_recipe() -> impl IntoView {
    let (endpoint, set_endpoint) = signal::<String>("recipe/random".to_string());
    let recipe = LocalResource::new(move || recipe::fetch(endpoint.get()));

    let error_fallback = move |errors: ArcRwSignal<Errors>| {
        let error_list = move || {
            errors.with(|errors| {
                errors
                    .iter()
                    .map(|(_, e)| view! { <li>{e.to_string()}</li> })
                    .collect::<Vec<_>>()
            })
        };

        view! {
            <div>
                <h2>"Error"</h2>
                <span class="error">{error_list}</span>
            </div>
        }
    };

    view! {
        <div><Transition fallback=|| view! { <div>"Loading..."</div> }>
            <ErrorBoundary fallback=error_fallback>
                {move || Suspend::new( async move {
                    recipe.map(|r| {
                        let r =  r.as_ref().unwrap();
                        let ingredients = r.ingredients
                            .lines()
                            .map(|line| view! { <li>{line.to_string()}</li> })
                            .collect::<Vec<_>>();
                        let instructions = r.instructions
                            .lines()
                            .map(|line| view! { <li>{line.to_string()}</li> })
                            .collect::<Vec<_>>();

                        view! {
                            <div class="recipe">
                                <h2>{r.title.clone()}</h2>
                                <h3>Ingredients</h3>
                                <ul>
                                    {ingredients}
                                </ul>
                                <h3>Instructions</h3>
                                <ol>
                                    {instructions}
                                </ol>
                            </div>
                            <div>
                                <p><strong>ID: </strong> {r.id.clone()}</p>
                                <p><strong>Tags: </strong> {format_tags(&r.tags)}</p>
                                <p>
                                    <strong>Source: </strong>
                                    <a href={r.source.clone()} target="_blank">{r.source.clone()}</a>
                                </p>
                            </div>
                        }
                    })
                })}
            </ErrorBoundary>
        </Transition></div>
        <div>
            <button on:click=move |_| {
                let ep = "recipe/random".to_string();
                set_endpoint.set(ep)
            }>Show a new recipe</button>
            <EnterInput set_endpoint=set_endpoint/>
        </div>
    }
}

pub fn main() {
    use tracing_subscriber::fmt;
    use tracing_subscriber_wasm::MakeConsoleWriter;

    fmt()
        .with_writer(
            // To avoid trace events in the browser from showing their
            // JS backtrace, which is very annoying, in my opinion
            MakeConsoleWriter::default()
                .map_trace_level_to(tracing::Level::DEBUG),
        )
        // For some reason, if we don't do this in the browser, we get
        // a runtime error.
        .without_time()
        .init();
    console_error_panic_hook::set_once();
    mount_to_body(fetch_recipe)
}
