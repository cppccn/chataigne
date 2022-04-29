Any contribution to the layer's repositories and to the main project are
welcomes!

# Versionning and user fonctionnalities

However, development shouldn't break the command line api. And so for the
library description. If there is a big restructuration for any approved reason,
use a versionning systeme as in `Cargo`, `Docker`, `Github workflows`...

To simplify the review of your code. Explain what you want to do with an
example and maybe a unit test. If your feature make sens for you, for us it's
not obvious. However, I'm talking for all the cappuccino team, if the feature
doesn't break anything, it would be certainly accepted after a review check. 

# Refactoring and write code

A big part of that code has been dumped in a few days. An minimal architecture
was described but with a lot of `todo` comments and that architecture can be
grandly improved. If you feel that the code can be simplier, better organized,
and most important: shorter, create a PR directly.

With further rereading and code review, deprecated todos will be removed and
the relevants will be changed into issues. Please if you see one todo and you
want to takle it, start a discussion with developers before. :-)

A common rules for development in that particular project, is to abuse of the
ideologie of peeking libraries. So if you see something that you can write with
less codes, thanks to a crate, use it!
