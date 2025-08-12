use nxsh_core::stream::{Stream, StreamData, StreamPipeline};

#[test]
fn object_pipeline_map_and_filter() {
    // Build initial JSON array stream
    let mut s = Stream::new(nxsh_core::stream::StreamType::Json);
    for i in 0..10 {
        let _ = s.write(StreamData::Json(serde_json::json!({"i": i})));
    }

    // Map: add field doubled = i*2; Filter: keep even i
    let mut p = StreamPipeline::new();
    p.add_stage(|st| {
        st.map(|d| match d {
            StreamData::Json(v) => {
                let mut obj = v.clone();
                if let Some(i) = obj.get("i").and_then(|x| x.as_i64()) {
                    obj["doubled"] = serde_json::json!(i * 2);
                }
                Ok(StreamData::Json(obj))
            }
            _ => Ok(d.clone()),
        })
    })
    .add_stage(|st| {
        st.filter(|d| match d {
            StreamData::Json(v) => v.get("i").and_then(|x| x.as_i64()).map(|i| i % 2 == 0).unwrap_or(false),
            _ => true,
        })
    });

    let out = p.execute(s).expect("pipeline");
    let items = out.collect().expect("collect");
    assert_eq!(items.len(), 5);
}


