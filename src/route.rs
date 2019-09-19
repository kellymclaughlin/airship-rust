#![allow(clippy::type_complexity)]

use std::collections::HashMap;

use base64;
use itertools::Itertools;
use radix_trie::Trie;

use crate::resource::Webmachine;

#[derive(Clone)]
pub enum BoundOrUnbound {
    Bound(String),
    Var(String),
    RestUnbound,
}

#[derive(Clone)]
pub struct Route(Vec<BoundOrUnbound>);

impl From<&str> for Route {
    fn from(route_str: &str) -> Self {
        let route_vec: Vec<BoundOrUnbound> =
            route_str
            .split("</>")
            .map(|part| part.trim())
            .map(|part| {
                if part.starts_with("::") && part.ends_with("::") {
                    let var = part.trim_matches(':').to_string();
                    BoundOrUnbound::Var(var)
                } else if part == "*" {
                    BoundOrUnbound::RestUnbound
                } else {
                    BoundOrUnbound::Bound(part.to_string())
                }
            })
            .collect();
        Route(route_vec)
    }
}

pub fn route_text(route: &Route) -> String {
    route
        .0
        .iter()
        .map(|part| bound_or_unbound_text(part))
        .intersperse(",".to_string())
        .collect::<Vec<_>>()
        .concat()
}

fn bound_or_unbound_text(bou: &BoundOrUnbound) -> String {
    match *bou {
        BoundOrUnbound::Bound(ref t) => t.clone(),
        BoundOrUnbound::Var(ref t) => String::from(":") + &t,
        BoundOrUnbound::RestUnbound => String::from("*")
    }
}

#[derive(Clone)]
pub struct RoutedResource<R>(pub Route, pub R);

#[derive(Clone)]
pub enum RouteLeaf<R> {
    RouteMatch(RoutedResource<R>, Vec<String>),
    RVar,
    RouteMatchOrVar(RoutedResource<R>, Vec<String>),
    Wildcard(RoutedResource<R>),
}

/// Turns the list of routes in a 'RoutingSpec' into a 'Trie' for efficient
/// routing
///
/// Routing trie creation algorithm
/// 1. Store full paths as keys up to first `var`
/// 2. Calculate Base64 encoding of the URL portion preceding the
///    `var` ++ "var" and use that as key for the next part of the
///    route spec.
/// 3. Repeat step 2 for every `var` encountered until the route
///    is completed and maps to a resource.
#[derive(Clone)]
pub struct RoutingSpec<'a, R>(pub Vec<(&'a str, R)>);
pub struct RoutingTrie<R>(pub Trie<String, RouteLeaf<R>>);

impl<'a, R> From<RoutingSpec<'a, R>> for RoutingTrie<R>
where
    R: Webmachine
{
    fn from(spec: RoutingSpec<R>) -> Self {
        // Convert the route string into a vector of `Route`s
        let routes: Vec<(Route, R)> =
            spec
            .0
            .into_iter()
            .map(|(route_str, res)| {
                (Route::from(route_str), res)
            })
            .collect();
        let leaves = routes.into_iter().flat_map(route_leaves).collect();

        RoutingTrie(to_trie(leaves))
    }
}

fn route_leaves<R>(
    route_pair: (Route, R)
) -> Vec<(String, RouteLeaf<R>)>
where
    R: Webmachine
{
    let (route, resource) = route_pair;
    let fold_acc = (String::new(), Vec::new(), Vec::new(), false);
    let fold_res = route.0.iter().fold(fold_acc, route_fold_fun);

    let (key, mut routes, vars, is_wild) = fold_res;
    let final_key = if key.is_empty() {
        String::from("/")
    } else {
        key
    };

    let final_leaf = if is_wild {
        RouteLeaf::Wildcard(RoutedResource::<R>(route, resource))
    } else {
        RouteLeaf::RouteMatch(RoutedResource::<R>(route, resource), vars)
    };

    routes.push((final_key, final_leaf));
    routes
}

fn route_fold_fun<R>(
    fold_acc: (String, Vec<(String, RouteLeaf<R>)>, Vec<String>, bool),
    bou: &BoundOrUnbound
) -> (String, Vec<(String, RouteLeaf<R>)>, Vec<String>, bool)
where
    R: Webmachine
{
    if !fold_acc.3 {
        match bou {
            BoundOrUnbound::Bound(ref t) => {
                let key = fold_acc.0;
                let part_key = key + "/" + t;

                (part_key, fold_acc.1, fold_acc.2, false)
            },
            BoundOrUnbound::Var(ref t) => {
                let key = fold_acc.0;
                let part_key_str = [&key, "var"].concat();
                let part_key = base64::encode(part_key_str.as_bytes());
                let mut routes = fold_acc.1;
                let mut vars = fold_acc.2;

                routes.push((key.clone(), RouteLeaf::RVar));
                vars.push(t.to_string());
                (part_key, routes, vars, false)
            },
            BoundOrUnbound::RestUnbound => {
                (fold_acc.0, fold_acc.1, fold_acc.2, true)
            }
        }
    } else {
        (fold_acc.0, fold_acc.1, fold_acc.2, true)
    }
}

fn to_trie<R>(
    route_leaves: Vec<(String, RouteLeaf<R>)>
) -> Trie<String, RouteLeaf<R>>
where
    R: Webmachine
{
    route_leaves
        .into_iter()
        .fold(Trie::new(), insert_or_replace)
}

fn insert_or_replace<R>(
    mut t: Trie<String, RouteLeaf<R>>,
    kv: (String, RouteLeaf<R>),
) -> Trie<String, RouteLeaf<R>>
where
    R: Webmachine
{
    let (key, new_value) = kv;
    match t.remove(&key) {
        Some(current_value) => {
            let merged_value = merge_values(current_value, new_value);
            t.insert(key, merged_value)
        },
        None => t.insert(key, new_value)
    };
    t
}

fn merge_values<R>(
    l1: RouteLeaf<R>,
    l2: RouteLeaf<R>
) -> RouteLeaf<R>
where
    R: Webmachine
{
    match (l1, l2) {
        (RouteLeaf::Wildcard(x), _) => RouteLeaf::Wildcard(x),
        (_, RouteLeaf::Wildcard(y)) => RouteLeaf::Wildcard(y),
        (RouteLeaf::RVar, RouteLeaf::RVar) => RouteLeaf::RVar,
        (RouteLeaf::RVar, RouteLeaf::RouteMatch(x, y)) => RouteLeaf::RouteMatchOrVar(x, y),
        (RouteLeaf::RouteMatch(_, _), RouteLeaf::RouteMatch(x, y)) => RouteLeaf::RouteMatch(x, y),
        (RouteLeaf::RouteMatch(x, y), RouteLeaf::RVar) => RouteLeaf::RouteMatchOrVar(x, y),
        (RouteLeaf::RouteMatchOrVar(_, _), RouteLeaf::RouteMatch(x, y)) => RouteLeaf::RouteMatchOrVar(x, y),
        (RouteLeaf::RouteMatchOrVar(x, y), _) => RouteLeaf::RouteMatchOrVar(x, y),
        (_, v) => v,
    }
}

// Represents the root resource (@/@) in a 'RoutingSpec'.
pub fn root() -> Route {
    Route(vec![])
}

// Captures a named in a route and adds it to the 'routingParams' hashmap under the provided 'Text' value. For example,
//
// @
//    "blog" '</>' 'var' "date" '</>' 'var' "post"
// @
//
// will capture all URLs of the form @\/blog\/$date\/$post@, and add @date@ and @post@ to the 'routingParams'
// contained within the resource this route maps to.
pub fn var(s: String) -> Route {
    Route(vec![BoundOrUnbound::Var(s)])
}

// Captures a wildcard route. For example,
pub fn star() -> Route {
    Route(vec![BoundOrUnbound::RestUnbound])
}

pub fn route<'a, R>(
    routes: &'a RoutingTrie<R>,
    path_info: String
) -> Option<(&'a RoutedResource<R>, (HashMap<String, String>, Vec<String>))>
where
    R: Webmachine
{
    let match_result = routes.0.prefix_match(&path_info);
    match_route(&routes.0, match_result, vec![], None)
}

fn dispatch_list(
    dispatch: Option<String>,
    matched: &str
) -> Vec<String> {
    let upd_dispatch = match dispatch {
        Some(path) => path + matched,
        None       => String::from("")
    };
    upd_dispatch.split('/').map(|s| s.to_string()).collect()
}

fn match_route<'a, R>(
    routes: &'a Trie<String, RouteLeaf<R>>,
    matched: Option<(Box<String>, &'a RouteLeaf<R>, Box<String>)>,
    mut params: Vec<String>,
    dispatch: Option<String>,
) -> Option<(&'a RoutedResource<R>, (HashMap<String, String>, Vec<String>))>
where
    R: Webmachine
{
    match matched {
        // Nothing even partially matched the route
        None => {
            None
        },

        // The matched key is also a prefix of other routes, but the entire path
        // matched so handle like a RouteMatch.
        Some((ref matched_prefix, RouteLeaf::RouteMatchOrVar(r, vars), ref rest)) if rest.is_empty() => {
            let dispatch_list = dispatch_list(dispatch, matched_prefix);
            let mut params_map = HashMap::new();
            let iter = vars.iter().zip(params.iter());
            iter.for_each(|(v, p)| {
                params_map.insert(v.clone(), p.clone());
            });
            Some((r, (params_map, dispatch_list)))
        },

        // The entire path matched so return the resource, params, and
        // dispatch path
        Some((ref matched_prefix, RouteLeaf::RouteMatch(r, vars), ref rest)) if rest.is_empty() => {
            let dispatch_list = dispatch_list(dispatch, matched_prefix);
            let mut params_map = HashMap::new();
            let iter = vars.iter().zip(params.iter());
            iter.for_each(|(v, p)| {
                params_map.insert(v.clone(), p.clone());
            });

            Some((r, (params_map, dispatch_list)))
        },

        Some((ref _matched, RouteLeaf::RouteMatch(_r, _vars), _)) =>
        // Part of the request path matched, but the trie value at the
        // matched prefix is not an RVar or RouteMatchOrVar so there is no
        // match.
            None,

        Some((ref _matched, RouteLeaf::RouteMatchOrVar(_r, _vars), ref _rest)) =>
        //  Part of the request path matched and the trie value at the
        //  matched prefix is a RouteMatchOrVar so handle it the same as if
        //  the value were RVar.
        //     matchRoute' routes (Just (matched, RVar, rest)) ps dsp
            None,

        Some((ref _matched, RouteLeaf::RVar, ref rest)) if rest.is_empty() =>
            None,

        Some((ref _matched, RouteLeaf::RVar, ref rest)) if rest.starts_with("//") =>
            None,

        Some((ref matched, RouteLeaf::RVar, ref rest)) if rest.starts_with('/') => {
        // Part of the request path matched and the trie value at the
        // matched prefix is a RVar so calculate the key for the next part
            // of the route and continue attempting to match.
            let encoded_match = base64::encode(&[&matched, "var"].concat());
            let next_key: String = [encoded_match,
                                    rest.trim_start_matches('/').trim_start_matches(|m| m != '/').to_string()].concat();

            let updated_dispatch = dispatch.or_else(|| Some(String::from("")));
//             paramVal = decodeUtf8 . BC8.takeWhile (/='/')
            //                        $ BC8.dropWhile (=='/') rest
            let mut trimmed_rest = rest.trim_start_matches('/').to_string();
            let slash_offset = trimmed_rest.find('/').unwrap_or_else(|| trimmed_rest.len());
            let param_val: String = trimmed_rest.drain(..slash_offset).collect();
            params.push(param_val);
            let match_result = routes.prefix_match(&next_key);
            match_route(&routes, match_result, params, updated_dispatch)
        },

        Some((ref _matched, RouteLeaf::RVar, ref _rest)) => {
            None
        },

        // Encountered a wildcard (star) value in the trie so it's a match
        Some((ref _matched, RouteLeaf::Wildcard(r), ref rest)) => {
             let trimmed_rest = rest.trim_start_matches('/').to_string();
            Some((r, (HashMap::new(), vec![trimmed_rest])))
        }
    }
}
