# Graph File System

Interacting with files in a graph-like way.

## Scenarios

jxo@DESKTOP-FAQCGGM:~/graph_filesystem$ gfs
jxo@DESKTOP-FAQCGGM:{ ~, graph_filesystem }$

jxo@DESKTOP-FAQCGGM:{ ~, graph_filesystem }$ pwd
{ home, jxo, graph_filesystem }

jxo@DESKTOP-FAQCGGM:{ a, b, c }$ cd +{ d } -{ a }
jxo@DESKTOP-FAQCGGM:{ b, c, d }$ 

jxo@DESKTOP-FAQCGGM:{ ~, graph_filesystem }$ ls
jxo@DESKTOP-FAQCGGM:{ ~, graph_filesystem }$ mk file.txt
jxo@DESKTOP-FAQCGGM:{ ~, graph_filesystem }$ ls
file.txt

`{ a, b, c, d }file.txt` exists
jxo@DESKTOP-FAQCGGM:{ a, b, c }$ lsa
... d
jxo@DESKTOP-FAQCGGM:{ a, b, c }$ ls
...

jxo@DESKTOP-FAQCGGM:{ a, b, c }$ mv file.txt +{ d } -{ a }
jxo@DESKTOP-FAQCGGM:{ a, b, c }$ mv file.txt { x, y, z }
jxo@DESKTOP-FAQCGGM:{ a, b, c }$ mv file.txt { x, y, z }/new_name.txt

jxo@DESKTOP-FAQCGGM:{ a, b, c }$ rm file.txt

//
//
//

{ _tag_1_, _tag_2_ } _file_1_, _file_2_
{ com.jxo.gfs, terraform } main.tf, variables.tf

{ _tag_1_, _tag_2_, _tag_3_ } _file_1_
{ com.jxo.gfs, terraform, docs } README.md

// Show current tags
lt
{ com.jxo.gfs, terraform, docs }

// Tab completion
// At { com.jxo.gfs }
// Pressing tab should show unique tags that are used with the current tags.
// E.g., { terraform, docs, src }

// Moving file or change current tags 
at terraform, docs // { com.jxo.gfs } to { com.jxo.gfs, terraform, docs }
st terraform, docs // { com.jxo.gfs, terraform, docs } to { com.jxo.gfs }

// Open file and close handle
file_handle = open("{ com.jxo.gfs, terraform, docs } README.md")
close(file_handle)

// Make tag and delete tag
// Tags implicitly get created and deleted at first use and last use.

// Programming language
import { _tag_1_, _tag_2_, _tag_3_ } _file_1_, _file_2_ 

///
///
/// YOU KNOW WHAT, WHEN YOU LOOK AT IT OBJECTIVELY, TAGGING FS IS ONLY DIFFERENT FROM
/// NORMAL FS IN THAT TAGGING THERE IS NO ORDER. NORMAL FS HAS ORDER.
/// E.G., /src/v4/api/events.py compared to { src, api, v4 } events.py
/// WHY NO ADD ADDITIONAL ATTRIBUTES TO NORMAL FS TO ADD TAGS, THEN YOU CAN HAVE BOTH. 
/// MANY TIMES ORDERING IS NOT DESIRED. SOMETIMES IT IS.
/// E.G., { src, api, v4 } events.py, /au/com/jxo
/// WELL, THE ORDERING IS MORE SO THAT USERS WILL SEE CERTAIN FILES/DIRS FIRST.
/// FUCK THIS PROJECT. TBH, IT PROBABLY IS NOT A BENIFIT. A GOOD EXERCISE ON THINKING OBJECTIVELY.
///
///

