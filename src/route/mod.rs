use std::collections::HashMap;

use hyper::Method::*;
use itertools::Itertools;
use radix_trie::Trie;
use resource::Webmachine;

enum BoundOrUnbound {
    Bound(String),
    Var(String),
    RestUnbound,
}

struct Route(Vec<BoundOrUnbound>);

fn route_text(&route: Route) -> String {
    route
        .0
        .iter()
        .map(|&part| boundOrUnboundText!(part))
        .intersperse(",".to_string())
        .collect::<Vec<_>>()
        .concat();
}

fn boundOrUnboundText(&bou: BoundOrUnbound) -> String {
    match bou {
        BoundOrUnbound::Bound(t) => t,
        BoundOrUnbound::Var(t) => String::from(":") + t,
        BoundOrUnbound::RestUnbound => String::from("*"),
    }
}

// instance IsString Route where
//     fromString s = Route [Bound (fromString s)]

// data RoutedResource m
//     = RoutedResource Route (Resource m)
struct RoutedResource(Route, Webmachine);

enum RouteLeaf {
    RouteMatch(RoutedResource, Vec<String>),
    RVar,
    RouteMatchOrVar(RoutedResource, Vec<String>),
    Wildcard(RoutedResource),
}

/*
 * Turns the list of routes in a 'RoutingSpec' into a 'Trie' for efficient
 * routing
 */
pub fn run_router(routes: RoutingSpec) -> Trie<String, RouteLeaf> {
    to_trie(routes);
}

/*
 * Custom version of Trie.fromList that resolves key conflicts
 * in the desired manner. In the case of duplicate routes the
 * routes specified first are favored over any subsequent
 * specifications.
 */
fn to_trie(routes: RoutingSpec) -> Trie<String, RouteLeaf> {
    // L.foldl' insertOrReplace Trie.empty
    routes
        .iter()
        .fold(Trie::new(), |acc, x| insert_or_replace(acc, x));
}

fn insert_or_replace(
    t: Trie<String, RouteLeaf>,
    kv: (String, RouteLeaf),
) -> Trie<String, RouteLeaf> {
    // insertOrReplace t (k, v) =
    // let newV = maybe v (mergeValues v) $ Trie.lookup k t
    //     in Trie.insert k newV t
}

fn merge_values(l1: RouteLeaf, l2: RouteLeaf) -> RouteLeaf {
    match (l1, l2) {
        (Wildcard(x), _) => Wildcard(x),
        (_, Wildcard(y)) => Wildcard(y),
        (RVar, RVar) => RVar,
        (RVar, RouteMatch(x, y)) => RouteMatchOrVar(x, y),
        (RouteMatch(_, _), RouteMatch(x, y)) => RouteMatch(x, y),
        (RouteMatch(x, y), RVar) => RouteMatchOrVar(x, y),
        (RouteMatchOrVar(_, _), RouteMatch(x, y)) => RouteMatchOrVar(x, y),
        (RouteMatchOrVar(x, y), _) => RouteMatchOrVar(x, y),
        (_, v) => v,
    }
}

// -- | @a '</>' b@ separates the path components @a@ and @b@ with a slash.
// -- This is actually just a synonym for 'mappend'.
// (</>) :: Route -> Route -> Route
// (</>) = (<>)

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
    Route(vec![Var(t)])
}

// Captures a wildcard route. For example,
pub fn star() -> Route {
    Route(vec![RestUnbound])
}

/*
 * Routing trie creation algorithm
 * 1. Store full paths as keys up to first `var`
 * 2. Calculate Base64 encoding of the URL portion preceding the
 *    `var` ++ "var" and use that as key for the next part of the
 *    route spec.
 * 3. Repeat step 2 for every `var` encountered until the route
 *    is completed and maps to a resource.
 */
// (#>) :: MonadWriter [(B.ByteString, (RouteLeaf a))] m
//      => Route -> Resource a -> m ()
// k #> v = do
//     let (key, routes, vars, isWild) = foldl routeFoldFun ("", [], [], False) (getRoute k)
//         key' = if BC8.null key then "/"
//                else key
//         ctor = if isWild
//                   then Wildcard (RoutedResource k v)
//                   else RouteMatch (RoutedResource k v) vars
//     tell $ (key', ctor) : routes
//     where
//         routeFoldFun (kps, rt, vs, False) (Bound x) =
//             (B.concat [kps, "/", encodeUtf8 x], rt, vs, False)
//         routeFoldFun (kps, rt, vs, False) (Var x) =
//             let partKey = Base64.encode $ B.concat [kps, "var"]
//                 rt' = (kps, RVar) : rt
//             in (partKey, rt', x:vs, False)
//         routeFoldFun (kps, rt, vs, False) RestUnbound =
//             (kps, rt, vs, True)
//         routeFoldFun (kps, rt, vs, True) _ =
//             (kps, rt, vs, True)

// (#>=) :: MonadWriter [(B.ByteString, (RouteLeaf a))] m
//       => Route -> m (Resource a) -> m ()
// k #>= mv = mv >>= (k #>)

// Represents a fully-specified set of routes that map paths (represented as 'Route's) to 'Resource's.
//
//    myRoutes() -> RoutingSpec {
//      [
//          (root, myRootResource),
//          (["blog", var "date", var "post"], blogPostResource)
//          (["about"], aboutResource),
//          (["anything", star], wildcardResource)
//      ]
//    }
//
// newtype RoutingSpec m a = RoutingSpec {
//         getRouter :: Writer [(B.ByteString, RouteLeaf m)] a
//     } deriving ( Functor, Applicative, Monad
//                , MonadWriter [(B.ByteString, RouteLeaf m)]
//                )
// type RoutePair = <R: Webmachine>(String, r:R);
type RouteMatch = Option<(String, RouteLeaf, String)>;
type ResourceMatch = Option<(RoutedResource, (HashMap<String, String>, Vec<String>))>;
struct RoutingSpec(Vec<(String, Webmachine)>);

// route :: Trie (RouteLeaf a)
//       -> BC8.ByteString
//       -> Maybe (RoutedResource a, (HashMap Text Text, [Text]))
// route routes pInfo = let matchRes = Trie.match routes pInfo
//                      in matchRoute' routes matchRes mempty Nothing
fn route(
    routes: Trie<String, RouteLeaf>,
    pInfo: String,
) -> Option<(RoutedResource, (HashMap<String, String>, Vec<String>))> {
    let matchRes = Trie.lookup(routes, pInfo);
    matchRoute(routes, matchRes, vec![], None);
}

fn matchRoute(
    routes: Trie<String, RouteLeaf>,
    matched: RouteMatch,
    params: Vec<String>,
    dispatch: Option<String>,
) -> ResourceMatch {
    None;
}
// matchRoute' :: Trie (RouteLeaf a)
//             -> Maybe (B.ByteString, RouteLeaf a, B.ByteString)
//             -> [Text]
//             -> Maybe B.ByteString
//             -> Maybe (RoutedResource a, (HashMap Text Text, [Text]))
// matchRoute' _routes Nothing _ps _dsp =
//     -- Nothing even partially matched the route
//     Nothing
// matchRoute' routes (Just (matched, RouteMatchOrVar r vars, "")) ps dsp =
//     -- The matched key is also a prefix of other routes, but the
//     -- entire path matched so handle like a RouteMatch.
//     matchRoute' routes (Just (matched, RouteMatch r vars, "")) ps dsp
// matchRoute' _routes (Just (matched, RouteMatch r vars, "")) ps dsp =
//     -- The entire path matched so return the resource, params, and
//     -- dispatch path
//     Just (r, (fromList $ zip vars ps, dispatchList dsp matched))
//     where
//         dispatchList (Just d) m = toTextList $ B.concat [d, m]
//         dispatchList Nothing _ = mempty
//         toTextList bs = decodeUtf8 <$> BC8.split '/' bs
// matchRoute' _routes (Just (_matched, RouteMatch _r _vars, _)) _ps _dsp =
//     -- Part of the request path matched, but the trie value at the
//     -- matched prefix is not an RVar or RouteMatchOrVar so there is no
//     -- match.
//     Nothing
// matchRoute' routes (Just (matched, RouteMatchOrVar _r _vars, rest)) ps dsp =
//     -- Part of the request path matched and the trie value at the
//     -- matched prefix is a RouteMatchOrVar so handle it the same as if
//     -- the value were RVar.
//     matchRoute' routes (Just (matched, RVar, rest)) ps dsp
// matchRoute' routes (Just (matched, RVar, rest)) ps dsp
//     | BC8.null rest = Nothing
//     | BC8.take 2 rest == "//" = Nothing
//     | BC8.head rest == '/' =
//         -- Part of the request path matched and the trie value at the
//         -- matched prefix is a RVar so calculate the key for the next part
//         -- of the route and continue attempting to match.
//         let nextKey = B.concat [ Base64.encode $ B.concat [matched, "var"]
//                                , BC8.dropWhile (/='/') $ BC8.dropWhile (=='/') rest
//                                ]
//             updDsp = if isNothing dsp then Just mempty
//                      else dsp
//             paramVal = decodeUtf8 . BC8.takeWhile (/='/')
//                        $ BC8.dropWhile (=='/') rest
//             matchRes = Trie.match routes nextKey
//         in matchRoute' routes matchRes (paramVal:ps) updDsp
//     | otherwise = Nothing
// matchRoute' _routes (Just (_matched, Wildcard r, rest)) _ps _dsp =
//     -- Encountered a wildcard (star) value in the trie so it's a match
//     Just (r, (mempty, decodeUtf8 <$> [BC8.dropWhile (=='/') rest]))
