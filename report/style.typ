#let assignment-title(
  title: [title],
  sub1: [],
  sub2: [],
  ext1: [],
	ext2: [],
	inset: (left: 8pt, bottom: 8pt),
	fontsize: 14pt
) = {
  let hasextra = ext1 != [] or ext2 != []
  let doubledeck = sub2 != [] or ext2 != []
  let fields = (
    (
      table.cell(
        colspan: if (hasextra) { 2 } else { 1 }, inset: inset,
        {
          set text(20pt)
          title; h(1fr)
        }
      ),
      table.hline(),
      { set text(fontsize); sub1; h(1fr) }
    ),

    if (hasextra) {
      set text(fontsize); ext1; h(1fr)
    } else { () },

    if (doubledeck) { (
      { set text(fontsize); sub2; h(1fr) }
    ) } else { () },

    if (hasextra) {
      set text(fontsize); ext2; h(1fr)
    } else { () },
  ).flatten()

  table(
    columns: if (hasextra) { 2 } else {1},
    align: (left, left),
    inset: (
      left: inset.left,
      bottom: if (doubledeck) { 3pt } else { 0pt },
      top: 8pt
    ),
    stroke: (left: 1pt, top: none, right: none, bottom: none),
    ..fields
  )
}

#let assignment-title-rule(
  header: [header],
	title: [title],
  sub1: [],
  sub2: [],
  ext1: [],
	ext2: [],
	inset: (left: 8pt, bottom: 8pt),
	fontsize: 14pt,
  pagenum: none,
) = doc => {
  set page(
    "a4",
    margin: (x: 0.5in, top: 0.8in, bottom: 0.5in),
    header: if (header == none) { none } else {
      box(
        stroke: (left: none, right: none, top: none, bottom: .7pt),
        inset: (bottom: 5pt),
        header
      )
    },
    numbering: pagenum
  )
  set text(size: 12pt, font: "TeX Gyre Schola")
  set heading(numbering: "1.")
  set par(justify: true)
  assignment-title(
    title: title,
    sub1: sub1,
    sub2: sub2,
    ext1: ext1,
    ext2: ext2,
    inset: inset,
    fontsize: fontsize
  )
  // show: word-count

  doc
}
