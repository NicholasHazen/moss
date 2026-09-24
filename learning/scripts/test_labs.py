import unittest

from labs import Examples, checked_source, examples, test_names


class LabIsolation(unittest.TestCase):
    def test_learner_cannot_remove_required_assertions(self):
        reference = examples()[("data-runnable", "ownership")]
        learner = reference.split("#[cfg(test)]")[0] + "#[cfg(test)] mod tests {}"
        checked = checked_source(reference, learner)
        self.assertEqual(test_names(checked), {
            "tests::capture_preserves_the_source",
            "tests::captured_reserve_is_independent",
            "tests::captured_text_is_independent",
        })
        self.assertIn('assert_eq!(reading.name, "Fern")', checked)

    def test_learner_implementation_is_the_one_checked(self):
        reference = examples()[("data-runnable", "ownership")]
        starter = examples()[("data-starter", "ownership")]
        checked = checked_source(reference, starter)
        self.assertIn('todo!("save owned text and a copied reserve")', checked)
        self.assertEqual(test_names(checked), test_names(reference))

    def test_html_entities_are_decoded_without_changing_source(self):
        parser = Examples()
        parser.feed('<pre><code data-runnable="x">fn f(x: &amp;str) -&gt; bool { x == "&lt;" }</code></pre>')
        self.assertEqual(parser.blocks[("data-runnable", "x")], 'fn f(x: &str) -> bool { x == "<" }')

    def test_ambiguous_reference_is_rejected(self):
        with self.assertRaises(ValueError):
            checked_source("#[cfg(test)] x #[cfg(test)] y", "fn main() {}")


if __name__ == "__main__":
    unittest.main()
