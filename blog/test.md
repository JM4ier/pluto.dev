---
title: Rendering Test Page
published: true
version: 3
---

# Test page

On this website I will maybe publish some stuff, but for now this just serves as test for the markdown to html conversion.

---
## Heading 2
### Heading 3
#### Heading 4

**bold text**

*cursive text*

- list entry
- another list entry 
- the third list entry

1. is this a numbered list?
2. test
3. test

[Here is a link to the site itself.](.)

## This is some of the code that produces this site:

```rs
use pulldown_cmark::*;

mod code;

fn main() {
   let markdown_input = include_str!("content.md");
   let parser = Parser::new(markdown_input);

   let mut code_lang = None;

   use Event::*;
   let parser = parser.flat_map(|event| {
      eprintln!("{:?}", event);
      match &event {
         Start(Tag::CodeBlock(kind)) => {
            if let CodeBlockKind::Fenced(kind) = kind {
               code_lang = Some(kind.clone().into_string());
            }
            vec![Html(r#"<div class="code">"#.into()), event]
         }
         Text(code) => code_lang
            .as_ref()
            .map(|lang| code::highlight(&code, &lang).ok())
            .flatten()
            .map_or(vec![event], |html| vec![Html(html.into())]),
         End(Tag::CodeBlock(_)) => {
            code_lang = None;
            vec![event, Html("</div>".into())]
         }
         _ => {
            vec![event]
         }
      }
   });

   let mut html_out = String::new();
   html::push_html(&mut html_out, parser);

   println!(
      include_str!("skeleton.html"),
      body = html_out,
      title = "Website"
   );
}
```

Now for some different code:

```cs
using System.Collections;
using System.Collections.Generic;
using UnityEngine;
using System.Linq;

public class DeathZone : MonoBehaviour
{

   void OnTriggerEnter2D (Collider2D collider)
   {
      // professional web design right here
      var resetObjects = collider.gameObject.GetComponentsInChildren<IReset>().ToList();
      resetObjects.AddRange(collider.gameObject.GetComponentsInParent<IReset>());
      foreach(var obj in resetObjects)
      {
         obj.Reset();
      }
   }
}
```

