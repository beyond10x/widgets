---
format: aep.planning-md/1
id: story:phonebook
kind: story
status: implemented
title: Hold a phonebook
summary: 'softphone.directory: contacts owning their addresses, and a name on a live call tile.'
relations:
- decomposes: epic:browser-softphone
- depends_on: story:media-session-domain
revision: 5
---
# Hold a phonebook

## Outcome

`softphone.directory` holds contacts and the addresses they are reachable at, and a live call tile
can show a name instead of a number.

## Acceptance

`ess validate` accepts the domain. A `ContactAddress.address` is exactly the type
`softphone.control.Dial` takes, so nothing converts between choosing a contact and calling one.
`softphone.control` names no contact — search the compiled `domains."softphone.control"` for
`softphone.directory` and it returns nothing.

**What `owns` buys, stated correctly.** This story shipped claiming "deleting a contact cannot orphan
an address: the relation is `owns`". That was wrong, and the domain's own comment said so eight lines
in: what a delete *does* is a command outcome, not a property of the relation. `DeleteContact` moves
`Contact.delete` and touches no address, so three `Active` addresses can point at a `Deleted` contact.

What `owns` actually buys is three refusals at `ess validate`: a second entity claiming to own
`ContactAddress`, a `via` field that is missing or mistyped, and a target nothing declares. Removing
the addresses is a host obligation, and it is one of the 68 on the *Obligations* page rather than a
guarantee.

`Contacts` and `ContactAddresses` now filter on `state == Active`, so a deleted contact stops
appearing in the phonebook even while the row survives.

## What was built

`ess/domains/directory.yaml`. `Contact` — `contact_id`, `display_name`. `ContactAddress` —
`address_id`, `contact_id`, `address: softphone.control.RemoteAddress`, `label`. Five commands:
`AddContact`, `RenameContact`, `AddAddress`, `RemoveAddress`, `DeleteContact`. Three views.

Bridge 5 is the `owns`/many entry on `Contact`, carried by `contact_id` on the address — the
canonical shape from the relations design document §2.1, and the reason the compiler can refuse a
second owner at all.

Bridge 8 is on `softphone.presentation.CallTile`, not here: the tile is the thing that wants a name,
so the tile is what references a contact. That keeps name resolution out of the control surface.

## One grant that is a statement, not an omission

`softphone.directory` declares `Human` alone. An agent dials the address it was given —
`softphone.control.Dial` takes a `RemoteAddress`, never a contact — so it needs no grant here.
Widening this to an agent is a decision somebody should take deliberately, not a gap to fill in
passing.

## Not built

Contact import or export, and any address book the browser did not receive by hand.
