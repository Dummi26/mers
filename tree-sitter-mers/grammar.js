module.exports = grammar({
  name: 'mers',
  rules: {
    source_file: $ => repeat($.definition),
    definition: $ => choice(
      $.init,
      $.assign,
      $.if,
      $.func,
      $.block,
      $.tuple,
      $.chain,
      $.string,
      $.number,
      $.variable,
    ),
    definition_in_chain: $ => choice(
      $.block,
      $.tuple,
      $.string,
      $.number,
      $.variable,
    ),
    definition_initable: $ => choice(
      $.variable,
      $.tuple,
    ),
    definition_assignable: $ => choice(
      $.variable,
      $.tuple,
      $.block,
      $.string,
      $.number,
    ),

    block: $ => seq(
      $.block_start,
      repeat($.definition),
      $.block_end
    ),
    block_start: $ => '{',
    block_end: $ => '}',

    if: $ => prec.left(seq(
      $.if_if,
      $.definition,
      $.definition,
      optional(seq(
        $.if_else,
        $.definition
      ))
    )),
    if_if: $ => 'if',
    if_else: $ => 'else',

    func: $ => seq(
      $.func_arg,
      $.func_arrow,
      $.func_body
    ),
    func_arg: $ => $.definition_assignable,
    func_arrow: $ => '->',
    func_body: $ => $.definition,

    init: $ => seq(
      $.init_to,
      $.init_colonequals,
      $.init_source,
    ),
    init_to: $ => $.definition_initable,
    init_colonequals: $ => ':=',
    init_source: $ => $.definition,

    assign: $ => seq(
      $.assign_to,
      $.assign_equals,
      $.assign_source,
    ),
    assign_to: $ => $.definition_assignable,
    assign_equals: $ => '=',
    assign_source: $ => $.definition,

    tuple: $ => seq(
      $.tuple_start,
      repeat(seq(
        $.definition,
        $.tuple_separator
      )),
      $.tuple_end
    ),
    tuple_start: $ => '(',
    tuple_end: $ => ')',
    tuple_separator: $ => /(,\s*)|\s+/,

    chain: $ => seq(
      $.chain_dot,
      $.definition_in_chain,
    ),
    chain_dot: $ => '.',

    number: $ => /[\+-]?(\d+)|(\d+\.\d+)/,

    variable: $ => /&?[^\s:=\.\{\}\[\]\(\)\d"]+/,

    string: $ => seq(
      '"',
      $.string_content,
      '"'
    ),
    string_content: $ => /([^\\"]|[\\.])+/,

  }
})
