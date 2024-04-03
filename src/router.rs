#[layout(HeaderFooter)]
        #[route("/")]
        #[redirect("/platforms", || Route::Homepage {})]
        #[redirect("/platforms/web", || Route::Homepage {})]
        #[redirect("/platforms/desktop", || Route::Homepage {})]
        #[redirect("/platforms/liveview", || Route::Homepage {})]
        #[redirect("/platforms/mobile", || Route::Homepage {})]
        #[redirect("/platforms/ssr", || Route::Homepage {})]
        #[redirect("/platforms/tui", || Route::Homepage {})]
        Homepage {},

        #[route("/awesome")]
        Awesome {},

        #[route("/deploy")]
        Deploy {},

        #[route("/tutorials/:id")]
        Tutorial { id: usize },

        #[nest("/blog")]
            #[route("/")]
            BlogList {},
            #[route("/release-050")]
            PostRelease050 {},
            #[route("/templates-diffing")]
            PostTemplate {},
            #[route("/going-fulltime")]
            PostFulltime {},
            #[route("/release-040")]
            PostRelease040 {},
            #[route("/release-030")]
            PostRelease030 {},
            #[route("/release-020")]
            PostRelease020 {},
            #[route("/introducing-dioxus")]
            PostRelease010 {},
        #[end_nest]

        #[layout(Learn)]
            #[nest("/learn")]
                #[redirect("/", || Route::Docs { child: BookRoute::Index {} })]

                #[route("/0.3/:..segments")]
                DocsO3 {
                    segments: Vec<String>
                },
                #[route("/0.4/:..segments")]
                DocsO4 {
                    segments: Vec<String>
                },
                #[child("/0.5")]
                Docs { child: BookRoute },
            #[end_nest]
        #[end_layout]
    #[end_nest]
    #[redirect("/docs/0.3/:..segments", |segments: Vec<String>| Route::DocsO3 { segments })]
    #[redirect("/docs/:.._segments", |_segments: Vec<String>| Route::Docs { child: BookRoute::Index {} })]
    #[route("/:..segments")]
    #[route("/:..segments")]
    Err404 { segments: Vec<String> },
