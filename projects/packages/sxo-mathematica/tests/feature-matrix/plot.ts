import { feature } from '@sxo/harness';

export const plotFeatures = [
    feature('ListPlot', 'plot').unsupported().effectful().gap('listplot.basic', 'ListPlot[{{1, 1}, {2, 4}}]', { expected: '<svg' }).done(),
    feature('Plot3D', 'plot')
        .unsupported('juxtaposition x y may parse as sequence; no 3-D path')
        .effectful()
        .gap('plot3d.basic', 'Plot3D[x*y, {x, 0, 1}, {y, 0, 1}]', { expected: '<svg' })
        .done(),
    feature('Plot', 'plot')
        .partial(
            'SVG→PNG visual: curve+L-axes readable; missing tick labels, no Frame, not commercial MMA axes-at-origin. Negative domain {x,-1,1} via unary-minus fold',
        )
        .effectful()
        .plot('plot.square', 'Plot[x^2, {x, 0, 1}]', { expected: '<svg' })
        .plot('plot.sin', 'Plot[Sin[x], {x, 0, 6}]', { expected: '<svg' })
        .gap('plot.neg_domain', 'Plot[x^2, {x, -1, 1}]', { expected: '<svg', notes: 'currently not a supported 1-D plot form' })
        .done(),
    feature('ParametricPlot', 'plot')
        .unsupported()
        .effectful()
        .gap('parametric.circle', 'ParametricPlot[{Cos[t], Sin[t]}, {t, 0, 2}]', { expected: '<svg' })
        .done(),
    feature('ContourPlot', 'plot')
        .unsupported()
        .effectful()
        .gap('contour.circle', 'ContourPlot[x^2 + y^2, {x, -1, 1}, {y, -1, 1}]', { expected: '<svg' })
        .done(),
    feature('PolarPlot', 'plot').unsupported().effectful().gap('polarplot.1', 'PolarPlot[1, {t, 0, 2}]', { expected: '<svg' }).done(),
    feature('RegionPlot', 'plot')
        .unsupported()
        .effectful()
        .gap('regionplot.disk', 'RegionPlot[x^2 + y^2 < 1, {x, -1, 1}, {y, -1, 1}]', { expected: '<svg' })
        .done(),
    feature('BarChart', 'plot').unsupported().effectful().gap('barchart.3', 'BarChart[{1, 2, 3}]', { expected: '<svg' }).done(),
];
