# Grammar

$$
\begin{align}
    [\text{file}] &\to [\text{stmt}]* \\
    [\text{stmt}] &\to
    \begin{cases}
    [\text{ident}]([\text{parameters}]?) = [\text{expr}] \\
    [\text{ident}] = [\text{expr}] \\
    [\text{expr}] \\
    [\text{comment}]
    \end{cases} \\
    [\text{parameters}] &\to [\text{ident}] (, [\text{ident}])^* \\
    [\text{expr}] &\to \begin{cases}
        [\text{float\_literal}] \\
        [\text{ident}] \\
        [\text{expr}] + [\text{expr}] \\
        [\text{expr}] - [\text{expr}] \\
        [\text{expr}] * [\text{expr}] \\
        [\text{expr}]\space/\space[\text{expr}] \\
        [([\text{expr}] (, [\text{expr}])^*)?]
    \end{cases} \\
    [\text{from\_to\_as\_block}] &\to \text{from}\space[\text{expr}]\space\text{to}\space[\text{expr}]\space\text{as}\space[\text{ident}]( \text{with step}\space[\text{expr}])?\space\{ [stmt]^* \}
\end{align}
$$
