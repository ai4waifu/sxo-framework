//! Harness **load → bind → invoke** integration tests (JSON args, no program stitching).

use sxo_napi::HostSession;

#[test]
fn max_min_invoke() {
    let session = HostSession::new(Some("mathematica".into())).expect("session");
    session.evaluate_definition("maxT[] := Max[1, 7]".into(), None).expect("max def");
    session.evaluate_definition("minT[] := Min[1, 7]".into(), None).expect("min def");
    let max_out = session.invoke("maxT".into(), vec![], None).expect("max invoke");
    let min_out = session.invoke("minT".into(), vec![], None).expect("min invoke");
    assert_eq!(session.term_to_json(&max_out).expect("max json"), "7");
    assert_eq!(session.term_to_json(&min_out).expect("min json"), "1");
}

#[test]
fn bind_json_list_length_invoke() {
    let nums: Vec<i64> = (0..20).map(|i| i).collect();
    let json = serde_json::to_string(&nums).expect("encode");
    let session = HostSession::new(Some("mathematica".into())).expect("session");
    session.evaluate_definition("listLen[nums_] := Length[nums]".into(), None).expect("definition");
    session.bind_json("nums".into(), json).expect("bind");
    let out = session.invoke("listLen".into(), vec!["nums".into()], None).expect("invoke");
    let got = session.term_to_json(&out).expect("json");
    assert_eq!(got, "20");
}

#[test]
fn bind_json_large_list_two_sum_invoke() {
    let nums: Vec<i64> = (0..20).map(|i| i).collect();
    let json = serde_json::to_string(&nums).expect("encode");
    let def = r#"twoSum[nums_, target_] := Module[{n = Length[nums]},
  Do[
    Do[
      If[nums[[i]] + nums[[j]] == target, Return[{i - 1, j - 1}]],
      {j, i + 1, n}
    ],
    {i, 1, n - 1}
  ];
  Null
]"#;
    let session = HostSession::new(Some("mathematica".into())).expect("session");
    session.evaluate_definition(def.into(), None).expect("definition");
    session.bind_json("nums".into(), json).expect("bind");
    session.bind_json("target".into(), "19".into()).expect("bind target");
    let out = session
        .invoke("twoSum".into(), vec!["nums".into(), "target".into()], None)
        .expect("invoke");
    let got = session.term_to_json(&out).expect("json");
    assert_eq!(got, "[0,19]");
}

#[test]
fn bind_json_then_invoke_two_sum_wolfram() {
    let def = r#"twoSum[nums_, target_] := Module[{n = Length[nums]},
  Do[
    Do[
      If[nums[[i]] + nums[[j]] == target, Return[{i - 1, j - 1}]],
      {j, i + 1, n}
    ],
    {i, 1, n - 1}
  ];
  Null
]"#;
    let session = HostSession::new(Some("mathematica".into())).expect("session");
    session.evaluate_definition(def.into(), None).expect("definition");
    session.bind_json("nums".into(), "[3,3]".into()).expect("bind nums");
    session.bind_json("target".into(), "6".into()).expect("bind target");
    let out = session
        .invoke("twoSum".into(), vec!["nums".into(), "target".into()], None)
        .expect("invoke");
    let got = session.term_to_json(&out).expect("json");
    assert_eq!(got, "[0,1]");
}
