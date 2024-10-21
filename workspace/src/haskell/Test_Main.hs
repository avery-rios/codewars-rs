module Main (main) where

import Test.Hspec
import {test_module} (spec)
             
main :: IO ()
main = hspec spec
