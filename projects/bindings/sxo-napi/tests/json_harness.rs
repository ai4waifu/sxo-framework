//! Harness **load → bind → invoke** integration tests (JSON args, no program stitching).

use sxo_napi::HostSession;

#[test]
fn reverse_integer_palindrome_invoke() {
    let session = HostSession::new(Some("mathematica".into())).expect("session");
    session
        .evaluate_definition(
            "reverse[x_] := Module[{mi = -2147483648, mx = 2147483647, n = x, ans = 0},
  While[n != 0,
    If[ans < Quotient[mi - 9, 10] + 1 || ans > Quotient[mx, 10], Return[0]];
    ans = ans * 10 + Mod[n, 10];
    n = Quotient[n - Mod[n, 10], 10]
  ];
  ans
]".into(),
            None,
        )
        .expect("reverse def");
    session
        .evaluate_definition(
            "isPalindrome[x_] := If[x < 0 || (x != 0 && Mod[x, 10] == 0), False, IntegerDigits[x] === Reverse[IntegerDigits[x]]]".into(),
            None,
        )
        .expect("palindrome def");
    session.bind_json("x".into(), "123".into()).expect("bind");
    let rev = session.invoke("reverse".into(), vec!["x".into()], None).expect("reverse");
    assert_eq!(session.term_to_json(&rev).expect("reverse json"), "321");
    let pal = session.invoke("isPalindrome".into(), vec!["x".into()], None).expect("palindrome");
    assert_eq!(session.term_to_json(&pal).expect("palindrome json"), "false");
    session.bind_json("x".into(), "121".into()).expect("bind 121");
    let pal2 = session.invoke("isPalindrome".into(), vec!["x".into()], None).expect("palindrome 121");
    assert_eq!(session.term_to_json(&pal2).expect("palindrome 121 json"), "true");
}

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
