# Math rendering sample

Inline: the gradient $\nabla f(x) = (\partial_1 f, \dots, \partial_n f)$ and the
softmax $p_i = e^{z_i} / \sum_j e^{z_j}$.

A subscripted variable: $x_1 + y_2 = z_3$.

## Display math

Cross-entropy loss:

$$
\mathcal{L}(\theta) = - \sum_i y_i \log p_\theta(y_i \mid x_i)
$$

Matrix product:

$$
(AB)_{ij} = \sum_{k=1}^{n} A_{ik} B_{kj}
$$

Aligned environment:

$$
\begin{aligned}
(f \circ g)'(x) &= f'(g(x)) \cdot g'(x) \\
\frac{d}{dx} e^{x^2}  &= 2x e^{x^2}
\end{aligned}
$$

## Mixed with code

Inline `$x$` inside a code span should not render as math.

```python
def softmax(z):
    e = np.exp(z - z.max())
    return e / e.sum()
```

Display block after code:

$$
\sigma(z) = \frac{1}{1 + e^{-z}}
$$
