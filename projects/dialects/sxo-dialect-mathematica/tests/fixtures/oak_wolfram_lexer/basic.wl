(* Comprehensive Wolfram Language (Mathematica) Lexer Test *)

(* Basic Arithmetic & Numbers *)
1 + 2 * 3^4;
5! (* Factorial *)
Sqrt[25]
1.23
1.23*^10  (* Scientific notation *)
16^^A0F   (* Base 16 *)
2^^1010   (* Base 2 *)

(* Lists and Matrices *)
list = {1, 2, 3, 4};
matrix = {{1, 0}, {0, 1}};
Range[10]
Table[i^2, {i, 1, 10}]
IdentityMatrix[3]

(* Functions and Patterns *)
f[x_] := x^2 + 2x + 1;
g[x_, y_] := x + y;
h[x_Integer] := x!;
k[x_List] := Total[x];
m[x_?Positive] := Sqrt[x];
n[x_, y_:1] := x^y; (* Optional argument *)

(* Pure Functions *)
(#^2 &) /@ {1, 2, 3}
Function[{x, y}, x + y]
#1 + #2 &
Select[list, # > 2 &]
Map[f, list]
Apply[Plus, list]

(* Control Flow *)
If[x > 0, Print["Positive"], Print["Negative"]];
For[i = 0, i < 10, i++, Print[i]];
While[x < 100, x = 2*x];
Do[Print[n^2], {n, 5}];
Switch[x, 1, "one", 2, "two", _, "other"];
Which[x < 0, "neg", x == 0, "zero", True, "pos"];
Module[{local = 5}, local * 2];
Block[{x = 1}, f[x]];
With[{c = 10}, c * x];

(* Rules and Replacements *)
{x, x^2, y} /. x -> 3;
list //. {a___, x_, y_, b___} /; x > y :> {a, y, x, b};
ReplaceAll[x^2 + y^2, {x -> 1, y -> 2}];
ReplaceRepeated[list, {x_, y_} :> {y, x}];

(* String Operations *)
"String" <> "Concatenation";
StringLength["Hello"];
StringMatchQ["test", RegularExpression["^t.*"]];
StringJoin[{"a", "b", "c"}];
StringSplit["a,b,c", ","];

(* Graphics and Plotting *)
Plot[Sin[x], {x, 0, 2 Pi}];
Graphics[{Red, Disk[], Blue, Line[{{0,0}, {1,1}}]}];
Plot3D[Sin[x + y], {x, 0, Pi}, {y, 0, Pi}];
Show[plot1, plot2];

(* Symbolic Computation *)
D[x^n, x];
Integrate[1/(1+x^2), x];
Simplify[Sin[x]^2 + Cos[x]^2];
Solve[x^2 + 2x + 1 == 0, x];
DSolve[y'[x] == y[x], y[x], x];
Limit[Sin[x]/x, x -> 0];
Series[Exp[x], {x, 0, 5}];

(* Associations (Maps) *)
assoc = <|"a" -> 1, "b" -> 2, "c" -> 3|>;
assoc["a"];
Keys[assoc];
Values[assoc];
Lookup[assoc, "d", 0];
KeySort[assoc];

(* Slots and Patterns *)
_   (* Blank *)
__  (* BlankSequence *)
___ (* BlankNullSequence *)
x: _Integer (* Pattern Name *)
x_?Positive (* Pattern Test *)
Except[0]
Longest["a" ..]
Shortest["a" ..]

(* Attributes *)
SetAttributes[f, Listable];
ClearAttributes[f, Listable];
Attributes[Plus]

(* Input/Output *)
Import["data.csv"];
Export["plot.png", %];
ReadList["file.txt", Number];

(* Comments *)
(* Nested (* comments *) are supported *)
(* 
   Multi-line
   comment
*)
