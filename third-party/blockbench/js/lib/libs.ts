import $ from 'jquery'
import * as threejs from "three"
import * as FIK from './fik'
import Vue from 'vue/dist/vue.js'
import JSZip from 'jszip'
import Prism from 'prismjs'
import 'prismjs/components/prism-json'
import vSortable from 'vue-sortable'
import Sortable from 'sortablejs'
import {marked} from 'marked'
import DOMPurify from 'dompurify'

Vue.use(vSortable)
Vue.directive('sortable', {
    inserted: function (el, binding) {
        new Sortable(el, binding.value || {})
    }
})

const THREE = Object.assign({}, threejs);

export {
	THREE,
    $,
    $ as jQuery,
	FIK,
	Vue,
	JSZip,
	Prism,
	marked,
	DOMPurify,
}
const global = {
	THREE,
    jQuery: $,
    $,
	FIK,
	Vue,
	JSZip,
	Prism,
	marked,
	DOMPurify,
}
Object.assign(window, global);
