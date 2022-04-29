Any contribution to the layer's repositories and to the main project are
welcomes!

## Versionning and user fonctionnalities

However, development shouldn't break the command line api. And so for the
library description. If there is a big restructuration for any approved reason,
use a versionning systeme as in `Cargo`, `Docker`, `Github workflows`...

To simplify the review of your code. Explain what you want to do with an
example and maybe a unit test. If your feature make sens for you, for us it's
not obvious. However, I'm talking for all the cappuccino team, if the feature
doesn't break anything, it would be certainly accepted after a review check. 

You can also work into the official original layer to add libraries and
version. No worries if you just want to add your own library and expose
humbleness your work. It'll be accepted quick!

# Refactoring and PRs and issues in general

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

In general, you will be reject by the CI if you don't respect the coding style.
Abuse of the cargo fmt and clppy command. Warnings are not accepted. Write
documentation. Don't use global static variables without a good reason.
Classic recommendations of code development in brief.

If you see a typo, a missing documentation or an error of traduction,
you can both use PR and Issues to signal us a problem, we will really
appreciate that kind of help.

## Issues and discussions

If you want to fix a bug, explain first the usecase. Try to fix it and push a
PR directly. You can also just write an issue if you prefer.

Issues has to respect a least these three rules:
- good expaination of the use cases and what you tried.
- reproductible, for us to fix correctly.
- good looking, [markdown documentation here](https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax) :-)


In discussions, use links to explain the architecture/moifications you want and
why is it appropirated. Be generous in examples. Read the existing code and
write an argumentation that speak to everybody. Prefer to be at least as
specific as generic. That is also a good rule to follow when writing an issue.